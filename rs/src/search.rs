use crate::cosine::cosine;
use crate::db::open_db;
use crate::ollama;

const TOP_N: usize = 20;

struct ScoredRow {
    sim: f64,
    file: String,
    line: i64,
    heading: String,
    title: String,
}

/// Run semantic search: embed query, score all chunks, print top 20 as TSV.
pub fn run(notebook_dir: &str, query: &str) {
    if query.is_empty() {
        return;
    }

    let embeddings = match ollama::embed(&[query]) {
        Some(e) => e,
        None => {
            eprintln!("Embedding failed");
            std::process::exit(1);
        }
    };
    let qv = &embeddings[0];

    let conn = match open_db(notebook_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("DB error: {e}");
            std::process::exit(1);
        }
    };

    let mut stmt = conn
        .prepare("SELECT file, heading, line, title, embedding FROM chunks WHERE embedding IS NOT NULL")
        .unwrap();

    let mut scored: Vec<ScoredRow> = stmt
        .query_map([], |row| {
            let file: String = row.get(0)?;
            let heading_raw: String = row.get(1)?;
            let line: i64 = row.get(2)?;
            let title: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
            let emb_json: String = row.get(4)?;
            Ok((file, heading_raw, line, title, emb_json))
        })
        .unwrap()
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

    for row in scored.iter().take(TOP_N) {
        println!(
            "{:.3}\t{}\t{}\t{}\t{}",
            row.sim, row.file, row.line, row.heading, row.title
        );
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
