use crate::cosine::cosine;
use crate::db::open_db;
use crate::ollama;

const TOP_N: usize = 20;

pub struct ScoredRow {
    pub sim: f64,
    pub file: String,
    pub line: i64,
    pub heading: String,
    pub title: String,
}

/// Semantic search: embed query, score all chunks, return sorted results.
pub fn search(notebook_dir: &str, query: &str) -> Result<Vec<ScoredRow>, String> {
    if query.is_empty() {
        return Ok(vec![]);
    }

    let embeddings = ollama::embed(&[query]).ok_or("Embedding failed")?;
    let qv = &embeddings[0];

    let conn = open_db(notebook_dir).map_err(|e| format!("DB error: {e}"))?;

    let mut stmt = conn
        .prepare("SELECT file, heading, line, title, embedding FROM chunks WHERE embedding IS NOT NULL")
        .map_err(|e| format!("Query error: {e}"))?;

    let mut scored: Vec<ScoredRow> = stmt
        .query_map([], |row| {
            let file: String = row.get(0)?;
            let heading_raw: String = row.get(1)?;
            let line: i64 = row.get(2)?;
            let title: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
            let emb_json: String = row.get(4)?;
            Ok((file, heading_raw, line, title, emb_json))
        })
        .map_err(|e| format!("Query error: {e}"))?
        .filter_map(|r| r.ok())
        .filter_map(|(file, heading_raw, line, title, emb_json)| {
            let emb: Vec<f64> = serde_json::from_str(&emb_json).ok()?;
            let sim = cosine(qv, &emb);
            let heading = strip_heading_prefix(&heading_raw).to_string();
            Some(ScoredRow {
                sim,
                file,
                line,
                heading,
                title,
            })
        })
        .collect();

    scored.sort_by(|a, b| b.sim.partial_cmp(&a.sim).unwrap_or(std::cmp::Ordering::Equal));

    Ok(scored)
}

/// Run semantic search and print top 20 as TSV.
pub fn run(notebook_dir: &str, query: &str) {
    match search(notebook_dir, query) {
        Ok(scored) => {
            for row in scored.iter().take(TOP_N) {
                println!(
                    "{:.3}\t{}\t{}\t{}\t{}",
                    row.sim, row.file, row.line, row.heading, row.title
                );
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn strip_heading_prefix(line: &str) -> &str {
    let s = line.trim_start_matches('#');
    s.trim_start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_heading_prefix() {
        assert_eq!(strip_heading_prefix("## Title"), "Title");
        assert_eq!(strip_heading_prefix("### Deep"), "Deep");
        assert_eq!(strip_heading_prefix("plain"), "plain");
    }
}
