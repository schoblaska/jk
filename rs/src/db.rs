use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS chunks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file TEXT NOT NULL,
  heading TEXT NOT NULL,
  line INTEGER NOT NULL,
  title TEXT,
  text TEXT NOT NULL,
  embedding TEXT
);
CREATE INDEX IF NOT EXISTS idx_chunks_file ON chunks(file);
";

pub fn db_path(notebook_dir: &str) -> PathBuf {
    Path::new(notebook_dir).join(".zk").join("search.db")
}

pub fn open_db(notebook_dir: &str) -> Result<Connection> {
    let path = db_path(notebook_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
