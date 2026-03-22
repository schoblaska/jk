use crate::chunk::chunk_markdown;
use crate::db::open_db;
use crate::files::find_markdown_files;
use crate::frontmatter::{self, Frontmatter};
use crate::ollama;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

struct FileResult {
    rel_path: String,
    title: String,
    chunks: Vec<ChunkWithEmbedding>,
    frontmatter: Frontmatter,
    links: Vec<String>,
    is_journal: bool,
    modified_at: Option<String>,
}

struct ChunkWithEmbedding {
    heading: String,
    lnum: usize,
    text: String,
    embedding: Vec<f64>,
}

fn embed_file(notebook_dir: &str, rel_path: &str) -> Option<FileResult> {
    let full_path = Path::new(notebook_dir).join(rel_path);
    let content = fs::read_to_string(&full_path).ok()?;
    let chunks = chunk_markdown(&content);
    if chunks.is_empty() {
        return None;
    }

    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let embeddings = match ollama::embed(&texts) {
        Some(e) => e,
        None => {
            eprintln!("  Failed to embed {rel_path}");
            return None;
        }
    };

    let title = if chunks[0].heading.starts_with("# ") {
        chunks[0].heading[2..].to_string()
    } else {
        String::new()
    };

    eprintln!("  {}: {} chunks", rel_path, chunks.len());

    let result_chunks: Vec<ChunkWithEmbedding> = chunks
        .into_iter()
        .zip(embeddings.into_iter())
        .map(|(c, emb)| ChunkWithEmbedding {
            heading: c.heading,
            lnum: c.lnum,
            text: c.text,
            embedding: emb,
        })
        .collect();

    // Parse frontmatter and links
    let fm = frontmatter::parse_frontmatter(&content);
    let file_dir = Path::new(rel_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let links = frontmatter::extract_links(&content, &file_dir);
    let is_journal = frontmatter::is_journal(rel_path);

    // Get file modification time
    let modified_at = fs::metadata(&full_path)
        .and_then(|m| m.modified())
        .ok()
        .map(|t| {
            let secs = t
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Format as ISO 8601 date
            let days = secs / 86400;
            // Approximate: good enough for recency scoring
            let y = 1970 + days / 365;
            let rem = days % 365;
            let m = rem / 30 + 1;
            let d = rem % 30 + 1;
            format!("{y:04}-{m:02}-{d:02}")
        });

    Some(FileResult {
        rel_path: rel_path.to_string(),
        title,
        chunks: result_chunks,
        frontmatter: fm,
        links,
        is_journal,
        modified_at,
    })
}

/// Full reindex: embed all files, replace all chunks in DB.
pub fn full_reindex(notebook_dir: &str) {
    eprintln!("Full reindex of {notebook_dir}");

    let md_files = find_markdown_files(notebook_dir);
    eprintln!("Found {} files", md_files.len());

    let results: Vec<FileResult> = md_files
        .par_iter()
        .filter_map(|rel_path| embed_file(notebook_dir, rel_path))
        .collect();

    let conn = open_db(notebook_dir).expect("Failed to open DB");
    conn.execute("DELETE FROM chunks", []).unwrap();
    conn.execute("DELETE FROM files", []).unwrap();
    conn.execute("DELETE FROM links", []).unwrap();

    let mut total_chunks = 0usize;
    {
        let tx = conn.unchecked_transaction().unwrap();
        for r in &results {
            // Insert file metadata
            let tags_str = r.frontmatter.tags.join(" ");
            let date = r
                .frontmatter
                .date
                .as_deref()
                .or(r.modified_at.as_deref());
            tx.execute(
                "INSERT OR REPLACE INTO files (path, title, date, tags, is_journal, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![r.rel_path, r.title, date, tags_str, r.is_journal as i64, r.modified_at],
            ).unwrap();

            // Insert links
            for dst in &r.links {
                tx.execute(
                    "INSERT OR IGNORE INTO links (src, dst) VALUES (?1, ?2)",
                    rusqlite::params![r.rel_path, dst],
                )
                .unwrap();
            }

            // Insert chunks (FTS triggers handle chunks_fts)
            for ch in &r.chunks {
                let emb_json = serde_json::to_string(&ch.embedding).unwrap();
                tx.execute(
                    "INSERT INTO chunks (file, heading, line, title, text, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![r.rel_path, ch.heading, ch.lnum as i64, r.title, ch.text, emb_json],
                ).unwrap();
                total_chunks += 1;
            }
        }
        tx.commit().unwrap();
    }

    eprintln!(
        "Indexed {} chunks from {} files",
        total_chunks,
        results.len()
    );
}

/// Incremental reindex: re-embed only the specified files.
pub fn incremental_reindex(notebook_dir: &str, file_args: &[String]) {
    let conn = open_db(notebook_dir).expect("Failed to open DB");

    for file_arg in file_args {
        let abs = match fs::canonicalize(file_arg) {
            Ok(p) => p,
            Err(_) => {
                // File deleted - purge from index
                let rel = pathdiff_relative(notebook_dir, file_arg);
                conn.execute("DELETE FROM chunks WHERE file = ?1", [&rel])
                    .unwrap();
                conn.execute("DELETE FROM files WHERE path = ?1", [&rel])
                    .unwrap();
                conn.execute("DELETE FROM links WHERE src = ?1", [&rel])
                    .unwrap();
                eprintln!("  {rel}: purged");
                continue;
            }
        };
        let rel = abs
            .strip_prefix(notebook_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_arg.clone());

        let result = match embed_file(notebook_dir, &rel) {
            Some(r) => r,
            None => continue,
        };

        let tx = conn.unchecked_transaction().unwrap();

        // Clear old data for this file
        tx.execute("DELETE FROM chunks WHERE file = ?1", [&rel])
            .unwrap();
        tx.execute("DELETE FROM files WHERE path = ?1", [&rel])
            .unwrap();
        tx.execute("DELETE FROM links WHERE src = ?1", [&rel])
            .unwrap();

        // Insert file metadata
        let tags_str = result.frontmatter.tags.join(" ");
        let date = result
            .frontmatter
            .date
            .as_deref()
            .or(result.modified_at.as_deref());
        tx.execute(
            "INSERT OR REPLACE INTO files (path, title, date, tags, is_journal, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![result.rel_path, result.title, date, tags_str, result.is_journal as i64, result.modified_at],
        ).unwrap();

        // Insert links
        for dst in &result.links {
            tx.execute(
                "INSERT OR IGNORE INTO links (src, dst) VALUES (?1, ?2)",
                rusqlite::params![result.rel_path, dst],
            )
            .unwrap();
        }

        // Insert chunks
        for ch in &result.chunks {
            let emb_json = serde_json::to_string(&ch.embedding).unwrap();
            tx.execute(
                "INSERT INTO chunks (file, heading, line, title, text, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![result.rel_path, ch.heading, ch.lnum as i64, result.title, ch.text, emb_json],
            ).unwrap();
        }
        tx.commit().unwrap();
    }
}

fn pathdiff_relative(base: &str, path: &str) -> String {
    Path::new(path)
        .strip_prefix(base)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}
