use crate::cosine::cosine;
use crate::db::open_db;
use crate::ollama;
use rayon::prelude::*;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

const W_SEMANTIC: f64 = 0.55;
const W_FTS: f64 = 0.30;
const W_TAG: f64 = 0.10;
const W_RECENCY: f64 = 0.05;

// Fallback weights when Ollama is unavailable
const W_FTS_FALLBACK: f64 = 0.85;
const W_TAG_FALLBACK: f64 = 0.10;
const W_RECENCY_FALLBACK: f64 = 0.05;

const CANDIDATE_LIMIT: usize = 50;
const LINK_SCORE_FACTOR: f64 = 0.6;
const EXCERPT_CHARS: usize = 300;

#[derive(Clone)]
pub struct RagResult {
    pub file: String,
    pub heading: String,
    pub title: String,
    pub score: f64,
    pub excerpt: String,
    pub date: Option<String>,
    pub tags: String,
    pub linked_from: Option<String>, // title of note that linked here
}

struct ParsedQuery {
    text: String,
    tags: Vec<String>,
}

struct ChunkCandidate {
    id: i64,
    file: String,
    heading: String,
    #[allow(dead_code)]
    line: i64,
    title: String,
    text: String,
    semantic_score: f64,
    fts_score: f64,
}

struct FileMeta {
    date: Option<String>,
    tags: String,
    is_journal: bool,
}

fn parse_query(raw: &str) -> ParsedQuery {
    let mut tags = Vec::new();
    let mut words = Vec::new();

    for token in raw.split_whitespace() {
        if let Some(tag) = token.strip_prefix('#') {
            if !tag.is_empty() {
                tags.push(tag.to_lowercase());
            }
        } else {
            words.push(token);
        }
    }

    ParsedQuery {
        text: words.join(" "),
        tags,
    }
}

fn semantic_search(conn: &Connection, query_embedding: &[f64]) -> Vec<ChunkCandidate> {
    let mut stmt = conn
        .prepare("SELECT id, file, heading, line, title, text, embedding FROM chunks WHERE embedding IS NOT NULL")
        .unwrap();

    let mut candidates: Vec<ChunkCandidate> = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let file: String = row.get(1)?;
            let heading: String = row.get(2)?;
            let line: i64 = row.get(3)?;
            let title: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
            let text: String = row.get(5)?;
            let emb_json: String = row.get(6)?;
            Ok((id, file, heading, line, title, text, emb_json))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .filter_map(|(id, file, heading, line, title, text, emb_json)| {
            let emb: Vec<f64> = serde_json::from_str(&emb_json).ok()?;
            let sim = cosine(query_embedding, &emb);
            Some(ChunkCandidate {
                id,
                file,
                heading: strip_heading_prefix(&heading).to_string(),
                line,
                title,
                text,
                semantic_score: sim,
                fts_score: 0.0,
            })
        })
        .collect();

    candidates.sort_by(|a, b| b.semantic_score.partial_cmp(&a.semantic_score).unwrap());
    candidates.truncate(CANDIDATE_LIMIT);
    candidates
}

fn fts_search(conn: &Connection, query_text: &str) -> Vec<ChunkCandidate> {
    if query_text.is_empty() {
        return vec![];
    }

    // Build FTS5 query: escape special chars, implicit AND
    let fts_query = query_text
        .split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ");

    let sql = "SELECT c.id, c.file, c.heading, c.line, c.title, c.text,
               bm25(chunks_fts, 5.0, 2.0, 1.0) AS fts_rank
               FROM chunks_fts
               JOIN chunks c ON c.id = chunks_fts.rowid
               WHERE chunks_fts MATCH ?1
               ORDER BY fts_rank
               LIMIT ?2";

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let results: Vec<ChunkCandidate> = match stmt.query_map(
        rusqlite::params![fts_query, CANDIDATE_LIMIT as i64],
        |row| {
            let id: i64 = row.get(0)?;
            let file: String = row.get(1)?;
            let heading: String = row.get(2)?;
            let line: i64 = row.get(3)?;
            let title: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
            let text: String = row.get(5)?;
            let fts_rank: f64 = row.get(6)?;
            Ok(ChunkCandidate {
                id,
                file,
                heading: strip_heading_prefix(&heading).to_string(),
                line,
                title,
                text,
                semantic_score: 0.0,
                fts_score: 1.0 / (1.0 + fts_rank.abs()),
            })
        },
    ) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => vec![],
    };
    results
}

fn load_file_meta(conn: &Connection) -> HashMap<String, FileMeta> {
    let mut stmt = conn
        .prepare("SELECT path, date, tags, is_journal FROM files")
        .unwrap();

    stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let date: Option<String> = row.get(1)?;
        let tags: String = row.get(2)?;
        let is_journal: bool = row.get::<_, i64>(3)? != 0;
        Ok((path, FileMeta { date, tags, is_journal }))
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

fn recency_score(date: Option<&str>, is_journal: bool) -> f64 {
    let date = match date {
        Some(d) => d,
        None => return 0.0,
    };

    let today = chrono_today();
    let days_old = days_between(date, &today).unwrap_or(365 * 5) as f64;
    let mut score = 1.0 / (1.0 + days_old / 365.0);
    if is_journal {
        score = (score * 1.5).min(1.0);
    }
    score
}

fn tag_boost(file_tags: &str, query_tags: &[String]) -> f64 {
    if query_tags.is_empty() {
        return 0.0;
    }
    let file_tags_lower: Vec<String> = file_tags
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .collect();
    let matched = query_tags
        .iter()
        .filter(|qt| file_tags_lower.iter().any(|ft| ft == *qt))
        .count();
    matched as f64 / query_tags.len() as f64
}

fn search_single_query(
    conn: &Connection,
    parsed: &ParsedQuery,
    file_meta: &HashMap<String, FileMeta>,
    ollama_available: bool,
) -> Vec<RagResult> {
    // Merge candidates from semantic + FTS into one map keyed by chunk id
    let mut candidates: HashMap<i64, ChunkCandidate> = HashMap::new();

    // Semantic search
    let (w_sem, w_fts, w_tag, w_rec) = if ollama_available {
        if let Some(embeddings) = ollama::embed(&[parsed.text.as_str()]) {
            let sem_results = semantic_search(conn, &embeddings[0]);
            for c in sem_results {
                candidates.insert(c.id, c);
            }
            (W_SEMANTIC, W_FTS, W_TAG, W_RECENCY)
        } else {
            (0.0, W_FTS_FALLBACK, W_TAG_FALLBACK, W_RECENCY_FALLBACK)
        }
    } else {
        (0.0, W_FTS_FALLBACK, W_TAG_FALLBACK, W_RECENCY_FALLBACK)
    };

    // FTS search
    let fts_results = fts_search(conn, &parsed.text);
    for c in fts_results {
        candidates
            .entry(c.id)
            .and_modify(|existing| {
                existing.fts_score = c.fts_score;
            })
            .or_insert(c);
    }

    // If query has tags but no text, also pull in all chunks from tag-matching files
    if parsed.text.is_empty() && !parsed.tags.is_empty() {
        let tag_files: HashSet<&str> = file_meta
            .iter()
            .filter(|(_, meta)| tag_boost(&meta.tags, &parsed.tags) > 0.0)
            .map(|(path, _)| path.as_str())
            .collect();

        if !tag_files.is_empty() {
            let mut stmt = conn
                .prepare("SELECT id, file, heading, line, title, text FROM chunks")
                .unwrap();
            let rows: Vec<_> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                })
                .unwrap()
                .filter_map(|r| r.ok())
                .filter(|(_, file, ..)| tag_files.contains(file.as_str()))
                .collect();

            for (id, file, heading, line, title, text) in rows {
                candidates.entry(id).or_insert(ChunkCandidate {
                    id,
                    file,
                    heading: strip_heading_prefix(&heading).to_string(),
                    line,
                    title: title.unwrap_or_default(),
                    text,
                    semantic_score: 0.0,
                    fts_score: 0.0,
                });
            }
        }
    }

    // Score all candidates
    let mut scored: Vec<RagResult> = candidates
        .into_values()
        .map(|c| {
            let meta = file_meta.get(&c.file);
            let tb = meta
                .map(|m| tag_boost(&m.tags, &parsed.tags))
                .unwrap_or(0.0);
            let rb = meta
                .map(|m| recency_score(m.date.as_deref(), m.is_journal))
                .unwrap_or(0.0);

            let score =
                w_sem * c.semantic_score + w_fts * c.fts_score + w_tag * tb + w_rec * rb;

            let excerpt = make_excerpt(&c.text);
            let tags = meta.map(|m| m.tags.clone()).unwrap_or_default();
            let date = meta.and_then(|m| m.date.clone());

            RagResult {
                file: c.file,
                heading: c.heading,
                title: c.title,
                score,
                excerpt,
                date,
                tags,
                linked_from: None,
            }
        })
        .collect();

    // Deduplicate: keep best score per (file, heading)
    let mut best: HashMap<(String, String), RagResult> = HashMap::new();
    for r in scored.drain(..) {
        let key = (r.file.clone(), r.heading.clone());
        match best.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if r.score > e.get().score {
                    e.insert(r);
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(r);
            }
        }
    }

    let mut results: Vec<RagResult> = best.into_values().collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

fn expand_links(
    conn: &Connection,
    top_results: &[RagResult],
    existing_files: &HashSet<String>,
    file_meta: &HashMap<String, FileMeta>,
) -> Vec<RagResult> {
    if top_results.is_empty() {
        return vec![];
    }

    let top_files: Vec<&str> = top_results
        .iter()
        .take(10)
        .map(|r| r.file.as_str())
        .collect();
    let top_scores: HashMap<&str, f64> = top_results
        .iter()
        .take(10)
        .map(|r| (r.file.as_str(), r.score))
        .collect();

    // Build title map for provenance
    let title_map: HashMap<&str, &str> = top_results
        .iter()
        .map(|r| (r.file.as_str(), r.title.as_str()))
        .collect();

    // Query bidirectional links
    let placeholders: String = top_files.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT DISTINCT dst AS linked, src AS via FROM links WHERE src IN ({placeholders})
         UNION
         SELECT DISTINCT src AS linked, dst AS via FROM links WHERE dst IN ({placeholders})"
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    // Bind parameters (top_files twice for UNION)
    let params: Vec<&dyn rusqlite::types::ToSql> = top_files
        .iter()
        .map(|f| f as &dyn rusqlite::types::ToSql)
        .chain(top_files.iter().map(|f| f as &dyn rusqlite::types::ToSql))
        .collect();

    let linked_files: Vec<(String, String)> = stmt
        .query_map(rusqlite::params_from_iter(&params), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .unwrap_or_else(|_| panic!("link query failed"))
        .filter_map(|r| r.ok())
        .filter(|(linked, _)| !existing_files.contains(linked))
        .collect();

    if linked_files.is_empty() {
        return vec![];
    }

    // For each linked file, fetch its title chunk (first chunk)
    let mut results = Vec::new();
    for (linked_file, via_file) in &linked_files {
        let mut stmt = conn
            .prepare("SELECT heading, line, title, text FROM chunks WHERE file = ?1 ORDER BY line LIMIT 1")
            .unwrap();
        let chunk = stmt
            .query_row([linked_file], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .ok();

        if let Some((heading, _line, title, text)) = chunk {
            let via_score = top_scores.get(via_file.as_str()).copied().unwrap_or(0.0);
            let score = LINK_SCORE_FACTOR * via_score;
            let via_title = title_map
                .get(via_file.as_str())
                .copied()
                .unwrap_or(via_file);

            let meta = file_meta.get(linked_file);
            let tags = meta.map(|m| m.tags.clone()).unwrap_or_default();
            let date = meta.and_then(|m| m.date.clone());

            results.push(RagResult {
                file: linked_file.clone(),
                heading: strip_heading_prefix(&heading).to_string(),
                title: title.unwrap_or_default(),
                score,
                excerpt: make_excerpt(&text),
                date,
                tags,
                linked_from: Some(via_title.to_string()),
            });
        }
    }

    // Deduplicate by file (keep best score)
    let mut best: HashMap<String, RagResult> = HashMap::new();
    for r in results {
        match best.entry(r.file.clone()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if r.score > e.get().score {
                    e.insert(r);
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(r);
            }
        }
    }

    best.into_values().collect()
}

/// Run the full RAG search pipeline.
pub fn search(
    notebook_dir: &str,
    raw_query: &str,
    limit: usize,
    do_expand_links: bool,
) -> Result<(Vec<RagResult>, bool), String> {
    if raw_query.trim().is_empty() {
        return Ok((vec![], true));
    }

    let conn = open_db(notebook_dir).map_err(|e| format!("DB error: {e}"))?;
    let file_meta = load_file_meta(&conn);

    // Check Ollama availability with a quick probe
    let ollama_available = ollama::embed(&["test"]).is_some();

    // Split comma-separated queries and run in parallel
    let queries: Vec<&str> = raw_query.split(',').map(|q| q.trim()).filter(|q| !q.is_empty()).collect();

    let parsed_queries: Vec<ParsedQuery> = queries.iter().map(|q| parse_query(q)).collect();

    // Run queries (parallel if multiple)
    let all_results: Vec<Vec<RagResult>> = if parsed_queries.len() == 1 {
        vec![search_single_query(&conn, &parsed_queries[0], &file_meta, ollama_available)]
    } else {
        // For multi-query, we need separate DB connections per thread
        parsed_queries
            .par_iter()
            .map(|pq| {
                let conn = open_db(notebook_dir).expect("DB error");
                let meta = load_file_meta(&conn);
                search_single_query(&conn, pq, &meta, ollama_available)
            })
            .collect()
    };

    // Merge results from all queries: dedup by (file, heading), keep max score
    let mut merged: HashMap<(String, String), RagResult> = HashMap::new();
    for results in all_results {
        for r in results {
            let key = (r.file.clone(), r.heading.clone());
            match merged.entry(key) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    if r.score > e.get().score {
                        e.insert(r);
                    }
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(r);
                }
            }
        }
    }

    let mut final_results: Vec<RagResult> = merged.into_values().collect();
    final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // One-hop link expansion
    if do_expand_links {
        let existing_files: HashSet<String> =
            final_results.iter().map(|r| r.file.clone()).collect();
        let mut link_results = expand_links(&conn, &final_results, &existing_files, &file_meta);
        final_results.append(&mut link_results);
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    }

    final_results.truncate(limit);
    Ok((final_results, ollama_available))
}

/// Format results as markdown for LLM consumption.
pub fn format_results(results: &[RagResult], raw_query: &str, ollama_available: bool) -> String {
    let mut out = String::new();

    if !ollama_available {
        out.push_str("> **Note:** Semantic search unavailable (Ollama not running). Results are fulltext-only.\n\n");
    }

    out.push_str(&format!("# Search: {raw_query}\n\n"));

    if results.is_empty() {
        out.push_str("No results found.\n");
        return out;
    }

    for r in results {
        // Header with provenance
        if let Some(ref via) = r.linked_from {
            out.push_str(&format!(
                "## [linked from: {}] {} [{:.2}]\n",
                via,
                display_title(r),
                r.score
            ));
        } else {
            out.push_str(&format!("## {} [{:.2}]\n", display_title(r), r.score));
        }

        // Metadata line
        let mut meta_parts = vec![format!("file: {}", r.file)];
        if let Some(ref d) = r.date {
            meta_parts.push(format!("date: {d}"));
        }
        if !r.tags.is_empty() {
            let tag_str: String = r
                .tags
                .split_whitespace()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join(" ");
            meta_parts.push(format!("tags: {tag_str}"));
        }
        out.push_str(&meta_parts.join(" | "));
        out.push('\n');

        // Excerpt
        if !r.excerpt.is_empty() {
            for line in r.excerpt.lines() {
                out.push_str(&format!("> {line}\n"));
            }
        }
        out.push('\n');
    }

    out
}

fn display_title(r: &RagResult) -> String {
    if !r.title.is_empty() && r.heading != r.title {
        format!("{} > {}", r.title, r.heading)
    } else if !r.title.is_empty() {
        r.title.clone()
    } else {
        r.heading.clone()
    }
}

fn make_excerpt(text: &str) -> String {
    // Skip the first line (it's the heading repeated with context prefix)
    let body = text
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    let body = body.trim();
    if body.len() <= EXCERPT_CHARS {
        body.to_string()
    } else {
        // Find a char boundary at or before EXCERPT_CHARS
        let mut end = EXCERPT_CHARS;
        while !body.is_char_boundary(end) {
            end -= 1;
        }
        // Cut at word boundary
        let truncated = &body[..end];
        match truncated.rfind(' ') {
            Some(pos) => format!("{}...", &truncated[..pos]),
            None => format!("{truncated}..."),
        }
    }
}

fn strip_heading_prefix(line: &str) -> &str {
    let s = line.trim_start_matches('#');
    s.trim_start()
}

/// Simple date math — days between two ISO dates. No chrono dependency.
fn chrono_today() -> String {
    // Use system date command for simplicity (avoids adding chrono crate)
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok();
    output
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "2026-01-01".to_string())
}

fn days_between(date_str: &str, today_str: &str) -> Option<i64> {
    let d = parse_date(date_str)?;
    let t = parse_date(today_str)?;
    Some((t - d).max(0))
}

fn parse_date(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;
    // Approximate days since epoch (good enough for relative comparison)
    Some(y * 365 + m * 30 + d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let p = parse_query("puglia #travel #food");
        assert_eq!(p.text, "puglia");
        assert_eq!(p.tags, vec!["travel", "food"]);
    }

    #[test]
    fn test_parse_query_no_tags() {
        let p = parse_query("machine learning models");
        assert_eq!(p.text, "machine learning models");
        assert!(p.tags.is_empty());
    }

    #[test]
    fn test_parse_query_only_tags() {
        let p = parse_query("#travel #italy");
        assert_eq!(p.text, "");
        assert_eq!(p.tags, vec!["travel", "italy"]);
    }

    #[test]
    fn test_tag_boost_full_match() {
        assert_eq!(tag_boost("travel italy", &["travel".into(), "italy".into()]), 1.0);
    }

    #[test]
    fn test_tag_boost_partial_match() {
        assert_eq!(tag_boost("travel", &["travel".into(), "food".into()]), 0.5);
    }

    #[test]
    fn test_tag_boost_no_match() {
        assert_eq!(tag_boost("coding", &["travel".into()]), 0.0);
    }

    #[test]
    fn test_tag_boost_no_query_tags() {
        assert_eq!(tag_boost("travel", &[]), 0.0);
    }

    #[test]
    fn test_recency_score_today() {
        let today = chrono_today();
        let score = recency_score(Some(&today), false);
        assert!(score > 0.9);
    }

    #[test]
    fn test_recency_score_old() {
        let score = recency_score(Some("2020-01-01"), false);
        assert!(score < 0.3);
    }

    #[test]
    fn test_recency_score_journal_boost() {
        let score_normal = recency_score(Some("2025-06-01"), false);
        let score_journal = recency_score(Some("2025-06-01"), true);
        assert!(score_journal >= score_normal);
    }

    #[test]
    fn test_recency_score_none() {
        assert_eq!(recency_score(None, false), 0.0);
    }

    #[test]
    fn test_make_excerpt_short() {
        let text = "## Heading\nShort body text here.";
        assert_eq!(make_excerpt(text), "Short body text here.");
    }

    #[test]
    fn test_make_excerpt_truncates() {
        let text = format!("## Heading\n{}", "word ".repeat(100));
        let excerpt = make_excerpt(&text);
        assert!(excerpt.len() <= EXCERPT_CHARS + 10); // some slack for "..."
        assert!(excerpt.ends_with("..."));
    }

    #[test]
    fn test_display_title_with_section() {
        let r = RagResult {
            file: "test.md".into(),
            heading: "Section A".into(),
            title: "My Note".into(),
            score: 0.5,
            excerpt: String::new(),
            date: None,
            tags: String::new(),
            linked_from: None,
        };
        assert_eq!(display_title(&r), "My Note > Section A");
    }

    #[test]
    fn test_display_title_same_as_heading() {
        let r = RagResult {
            file: "test.md".into(),
            heading: "My Note".into(),
            title: "My Note".into(),
            score: 0.5,
            excerpt: String::new(),
            date: None,
            tags: String::new(),
            linked_from: None,
        };
        assert_eq!(display_title(&r), "My Note");
    }

    #[test]
    fn test_days_between() {
        assert_eq!(days_between("2026-01-01", "2026-01-02"), Some(1));
        assert_eq!(days_between("2026-01-01", "2026-01-01"), Some(0));
    }

    #[test]
    fn test_format_results_empty() {
        let out = format_results(&[], "test query", true);
        assert!(out.contains("No results found"));
    }

    #[test]
    fn test_format_results_with_link() {
        let results = vec![RagResult {
            file: "a.md".into(),
            heading: "Section".into(),
            title: "Note A".into(),
            score: 0.8,
            excerpt: "Some text here".into(),
            date: Some("2026-01-01".into()),
            tags: "travel".into(),
            linked_from: Some("Note B".into()),
        }];
        let out = format_results(&results, "test", true);
        assert!(out.contains("[linked from: Note B]"));
        assert!(out.contains("file: a.md"));
        assert!(out.contains("#travel"));
    }
}
