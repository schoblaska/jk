use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: i64 = 2;

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

CREATE TABLE IF NOT EXISTS files (
  path TEXT PRIMARY KEY,
  title TEXT NOT NULL DEFAULT '',
  date TEXT,
  tags TEXT NOT NULL DEFAULT '',
  is_journal INTEGER NOT NULL DEFAULT 0,
  modified_at TEXT
);

CREATE TABLE IF NOT EXISTS links (
  src TEXT NOT NULL,
  dst TEXT NOT NULL,
  PRIMARY KEY (src, dst)
);
CREATE INDEX IF NOT EXISTS idx_links_dst ON links(dst);

CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
  title, heading, text,
  content='chunks', content_rowid='id',
  tokenize='porter unicode61'
);

CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
  INSERT INTO chunks_fts(rowid, title, heading, text)
  VALUES (new.id, new.title, new.heading, new.text);
END;

CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
  INSERT INTO chunks_fts(chunks_fts, rowid, title, heading, text)
  VALUES ('delete', old.id, old.title, old.heading, old.text);
END;
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
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version < SCHEMA_VERSION {
        // Drop old tables and rebuild (full reindex required)
        conn.execute_batch(
            "DROP TABLE IF EXISTS chunks_fts;
             DROP TRIGGER IF EXISTS chunks_ai;
             DROP TRIGGER IF EXISTS chunks_ad;
             DROP TABLE IF EXISTS chunks;
             DROP TABLE IF EXISTS files;
             DROP TABLE IF EXISTS links;",
        )?;
        conn.execute_batch(SCHEMA)?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    } else {
        conn.execute_batch(SCHEMA)?;
    }
    Ok(())
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

    #[test]
    fn migration_creates_all_tables() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        // Verify tables exist using query_row (execute doesn't work for SELECT on virtual tables)
        let _: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0)).unwrap();
        let _: i64 = conn.query_row("SELECT COUNT(*) FROM links", [], |r| r.get(0)).unwrap();
        let _: i64 = conn.query_row("SELECT COUNT(*) FROM chunks", [], |r| r.get(0)).unwrap();
        let _: i64 = conn.query_row("SELECT COUNT(*) FROM chunks_fts", [], |r| r.get(0)).unwrap();
    }

    #[test]
    fn fts_trigger_fires_on_insert() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "INSERT INTO chunks (file, heading, line, title, text) VALUES ('a.md', '# Test', 1, 'Test', 'hello world')",
            [],
        ).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks_fts WHERE chunks_fts MATCH 'hello'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn fts_trigger_fires_on_delete() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "INSERT INTO chunks (file, heading, line, title, text) VALUES ('a.md', '# Test', 1, 'Test', 'unique_word')",
            [],
        ).unwrap();
        conn.execute("DELETE FROM chunks WHERE file = 'a.md'", []).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks_fts WHERE chunks_fts MATCH 'unique_word'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
