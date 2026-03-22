use crate::{db, embed, rag};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::Parameters,
    model::*,
    schemars, tool, tool_box,
    transport::io::stdio,
};
use std::path::Path;

struct DateInfo {
    pretty: String, // "Fri, Mar 21 2026"
    iso: String,    // "2026-03-21"
}

fn today() -> Result<DateInfo, String> {
    let output = std::process::Command::new("date")
        .arg("+%a\t%b %d %Y\t%Y-%m-%d")
        .output()
        .map_err(|e| format!("date: {e}"))?;
    let s = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = s.trim().split('\t').collect();
    if parts.len() != 3 {
        return Err("date parse error".into());
    }
    Ok(DateInfo {
        pretty: format!("{}, {}", parts[0], parts[1]),
        iso: parts[2].to_string(),
    })
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RagSearchParams {
    /// Search query. Use #tag tokens for tag boosting (e.g. "puglia #travel").
    /// Comma-separated queries run in parallel (e.g. "italian cooking, wine regions").
    query: String,
    /// Maximum results to return (default: 15)
    #[serde(default)]
    limit: Option<usize>,
    /// Include one-hop linked notes in results (default: true)
    #[serde(default)]
    expand_links: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ReadNoteParams {
    /// One or more note paths relative to the notebook directory (e.g. ["ai/a1b2.md", "travel/c3d4.md"])
    paths: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateNoteParams {
    /// Title for the new note
    title: String,
    /// Optional markdown body content
    #[serde(default)]
    content: Option<String>,
    /// Optional space-separated tags without # prefix (default: "ai-generated")
    #[serde(default)]
    tags: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EditNoteParams {
    /// Path to the note relative to notebook directory (must be in ai/)
    path: String,
    /// New full content for the note (overwrites existing)
    content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RecentJournalsParams {
    /// Number of recent journal entries to return (default: 10)
    #[serde(default)]
    limit: Option<usize>,
    /// Which journals to return: "user" (default), "ai", or "all"
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AppendAiJournalParams {
    /// Markdown content to append to today's AI journal entry
    content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ReindexParams {
    /// Optional list of specific file paths to reindex incrementally. If empty, does a full reindex.
    #[serde(default)]
    files: Vec<String>,
}

#[derive(Clone)]
pub struct JkServer {
    notebook_dir: String,
}

impl JkServer {
    fn new(notebook_dir: String) -> Self {
        Self { notebook_dir }
    }

    #[tool(description = "Hybrid RAG search over notes. Combines semantic similarity, fulltext (BM25), tag boosting, and recency signals into a single ranked result set with excerpts. Use #tag tokens in the query to boost notes with matching tags. Comma-separated queries run in parallel and merge results. Returns markdown with scored results, metadata, and excerpts.")]
    async fn rag_search(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<RagSearchParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let query = params.query;
        let limit = params.limit.unwrap_or(20);
        let expand = params.expand_links.unwrap_or(true);
        tokio::task::spawn_blocking(move || {
            let (results, ollama_available) = rag::search(&dir, &query, limit, expand)?;
            Ok(rag::format_results(&results, &query, ollama_available, &dir))
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "List all notes in the notebook. Returns TSV: title, path")]
    async fn list_notes(&self) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        tokio::task::spawn_blocking(move || {
            let out = std::process::Command::new("zk")
                .args([
                    "list",
                    "--quiet",
                    "--format",
                    "{{title}}\t{{path}}",
                    "--sort",
                    "title",
                ])
                .env("ZK_NOTEBOOK_DIR", &dir)
                .current_dir(&dir)
                .output()
                .map_err(|e| format!("zk error: {e}"))?;
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "Read the full contents of one or more notes by their relative paths. Returns each note separated by a header line.")]
    async fn read_note(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<ReadNoteParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let paths = params.paths;
        tokio::task::spawn_blocking(move || {
            let base = Path::new(&dir)
                .canonicalize()
                .map_err(|e| format!("Invalid notebook: {e}"))?;
            let mut out = String::new();
            for (i, path) in paths.iter().enumerate() {
                if i > 0 {
                    out.push_str("\n---\n\n");
                }
                let full = Path::new(&dir).join(path);
                let canonical = match full.canonicalize() {
                    Ok(c) => c,
                    Err(e) => {
                        out.push_str(&format!("## {path}\nError: {e}\n"));
                        continue;
                    }
                };
                if !canonical.starts_with(&base) {
                    out.push_str(&format!("## {path}\nError: path outside notebook directory\n"));
                    continue;
                }
                if canonical.extension().and_then(|e| e.to_str()) != Some("md") {
                    out.push_str(&format!("## {path}\nError: only .md files can be read\n"));
                    continue;
                }
                match std::fs::read_to_string(&canonical) {
                    Ok(content) => {
                        if paths.len() > 1 {
                            out.push_str(&format!("## {path}\n\n"));
                        }
                        out.push_str(&content);
                    }
                    Err(e) => {
                        out.push_str(&format!("## {path}\nError: {e}\n"));
                    }
                }
            }
            Ok(out)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "List all tags used in the notebook")]
    async fn list_tags(&self) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        tokio::task::spawn_blocking(move || {
            let out = std::process::Command::new("zk")
                .args(["tag", "list", "--quiet"])
                .env("ZK_NOTEBOOK_DIR", &dir)
                .current_dir(&dir)
                .output()
                .map_err(|e| format!("zk error: {e}"))?;
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "Reindex the notebook: rebuilds zk index, regenerates index.md, and updates semantic embeddings. Pass specific file paths for incremental reindex, or omit for full reindex.")]
    async fn reindex(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<ReindexParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let files = params.files;
        tokio::task::spawn_blocking(move || {
            // zk index
            let _ = std::process::Command::new("zk")
                .args(["index", "--quiet"])
                .env("ZK_NOTEBOOK_DIR", &dir)
                .current_dir(&dir)
                .output();

            // zk gen-index
            let _ = std::process::Command::new("zk")
                .args(["gen-index"])
                .env("ZK_NOTEBOOK_DIR", &dir)
                .current_dir(&dir)
                .output();

            // Embed
            if files.is_empty() {
                embed::full_reindex(&dir);
                Ok("Full reindex complete.".to_string())
            } else {
                embed::incremental_reindex(&dir, &files);
                Ok(format!("Incremental reindex complete ({} files).", files.len()))
            }
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "Write new content to an existing note in the ai/ directory. Only ai/ notes can be edited — human notes are read-only. Overwrites the entire file.")]
    async fn edit_note(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<EditNoteParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let path = params.path;
        let content = params.content;
        tokio::task::spawn_blocking(move || {
            // Must be in ai/ subdirectory
            if !path.starts_with("ai/") {
                return Err("Only notes in ai/ can be edited. Human notes are read-only.".into());
            }
            let full = Path::new(&dir).join(&path);
            let canonical = full
                .canonicalize()
                .map_err(|e| format!("Invalid path: {e}"))?;
            let base = Path::new(&dir)
                .canonicalize()
                .map_err(|e| format!("Invalid notebook: {e}"))?;
            if !canonical.starts_with(&base) {
                return Err("Path outside notebook directory".into());
            }
            if canonical.extension().and_then(|e| e.to_str()) != Some("md") {
                return Err("Only .md files can be edited".into());
            }
            std::fs::write(&canonical, &content).map_err(|e| format!("Write error: {e}"))?;

            // Incremental reindex for the edited file
            let abs_str = canonical.to_string_lossy().to_string();
            embed::incremental_reindex(&dir, &[abs_str]);

            Ok(format!("Updated {path}"))
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "Get recent journal entries with full content, in reverse chronological order. Set source to \"user\" (default), \"ai\", or \"all\".")]
    async fn recent_journals(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<RecentJournalsParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let limit = params.limit.unwrap_or(10);
        let source = params.source.as_deref().unwrap_or("user");
        let source = source.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db::open_db(&dir).map_err(|e| format!("DB error: {e}"))?;

            // Both user (root/YYYY-MM-DD.md) and AI (ai/YYYY-MM-DD.md) journals
            // have is_journal=1. Distinguish by path prefix.
            let sql = match source.as_str() {
                "ai" => "SELECT path, date FROM files WHERE is_journal = 1 AND path LIKE 'ai/%' ORDER BY date DESC LIMIT ?1",
                "all" => "SELECT path, date FROM files WHERE is_journal = 1 ORDER BY date DESC LIMIT ?1",
                _ => "SELECT path, date FROM files WHERE is_journal = 1 AND path NOT LIKE 'ai/%' ORDER BY date DESC LIMIT ?1",
            };

            let mut stmt = conn.prepare(sql).map_err(|e| format!("Query error: {e}"))?;
            let entries: Vec<(String, String)> = stmt
                .query_map([limit as i64], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    ))
                })
                .map_err(|e| format!("Query error: {e}"))?
                .filter_map(|r| r.ok())
                .collect();

            if entries.is_empty() {
                return Ok(format!("No {source} journal entries found. Run `reindex` if the notebook has journal files."));
            }

            let mut out = String::new();
            for (i, (path, date)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push_str("\n---\n\n");
                }
                out.push_str(&format!("## {date} ({path})\n\n"));
                let full = Path::new(&dir).join(path);
                match std::fs::read_to_string(&full) {
                    Ok(content) => out.push_str(&content),
                    Err(e) => out.push_str(&format!("Error reading {path}: {e}\n")),
                }
            }
            Ok(out)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "Append content to today's AI journal entry (ai/YYYY-MM-DD.md). Creates the file with frontmatter if it doesn't exist. Use for session logs, observations, research notes.")]
    async fn append_ai_journal(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<AppendAiJournalParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let content = params.content;
        tokio::task::spawn_blocking(move || {
            let date = today()?;
            let ai_dir = Path::new(&dir).join("ai");
            std::fs::create_dir_all(&ai_dir).map_err(|e| format!("mkdir: {e}"))?;

            let rel = format!("ai/{}.md", date.iso);
            let abs_path = ai_dir.join(format!("{}.md", date.iso));

            if abs_path.exists() {
                // Append to existing
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&abs_path)
                    .map_err(|e| format!("open: {e}"))?;
                write!(file, "\n\n{content}").map_err(|e| format!("write: {e}"))?;
            } else {
                // Create with frontmatter
                let note = format!(
                    "# AI Journal: {}\ndate: [{}](../{})\ntags: #ai-journal\n\n{content}\n",
                    date.pretty, date.pretty, date.iso,
                );
                std::fs::write(&abs_path, &note).map_err(|e| format!("write: {e}"))?;

                // Register with zk
                let _ = std::process::Command::new("zk")
                    .args(["index", "--quiet"])
                    .env("ZK_NOTEBOOK_DIR", &dir)
                    .current_dir(&dir)
                    .output();
                let _ = std::process::Command::new("zk")
                    .args(["gen-index"])
                    .env("ZK_NOTEBOOK_DIR", &dir)
                    .current_dir(&dir)
                    .output();
            }

            // Incremental reindex
            let abs_str = abs_path.to_string_lossy().to_string();
            embed::incremental_reindex(&dir, &[abs_str]);

            Ok(rel)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    tool_box!(JkServer {
        rag_search,
        list_notes,
        read_note,
        edit_note,
        list_tags,
        create_note,
        reindex,
        recent_journals,
        append_ai_journal,
    });

    #[tool(description = "Create a new note in the ai/ directory. Returns the relative path of the created note.")]
    async fn create_note(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<CreateNoteParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        tokio::task::spawn_blocking(move || {
            let ai_dir = Path::new(&dir).join("ai");
            std::fs::create_dir_all(&ai_dir).map_err(|e| format!("mkdir: {e}"))?;

            // Random 4-char hex ID
            let id = loop {
                let mut buf = [0u8; 2];
                {
                    use std::io::Read;
                    std::fs::File::open("/dev/urandom")
                        .and_then(|mut f| f.read_exact(&mut buf))
                        .map_err(|e| format!("random: {e}"))?;
                }
                let candidate = format!("{:02x}{:02x}", buf[0], buf[1]);
                if !ai_dir.join(format!("{candidate}.md")).exists() {
                    break candidate;
                }
            };

            let date = today()?;
            let pretty = &date.pretty;
            let iso = &date.iso;

            // Tags
            let tag_str = match &params.tags {
                Some(t) => t
                    .split_whitespace()
                    .map(|t| format!("#{t}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                None => "#ai-generated".into(),
            };

            // Build note
            let mut note = format!(
                "# {}\ndate: [{}](../{})\ndescription:\ntags: {}\n",
                params.title, pretty, iso, tag_str,
            );
            if let Some(content) = &params.content {
                note.push('\n');
                note.push_str(content);
                note.push('\n');
            }

            let rel = format!("ai/{id}.md");
            let abs_path = ai_dir.join(format!("{id}.md"));
            std::fs::write(&abs_path, &note)
                .map_err(|e| format!("write: {e}"))?;

            // Reindex: update zk index, regenerate index.md, embed new note
            let _ = std::process::Command::new("zk")
                .args(["index", "--quiet"])
                .env("ZK_NOTEBOOK_DIR", &dir)
                .current_dir(&dir)
                .output();

            let _ = std::process::Command::new("zk")
                .args(["gen-index"])
                .env("ZK_NOTEBOOK_DIR", &dir)
                .current_dir(&dir)
                .output();

            let abs_str = abs_path.to_string_lossy().to_string();
            embed::incremental_reindex(&dir, &[abs_str]);

            Ok(rel)
        })
        .await
        .map_err(|e| e.to_string())?
    }
}

impl ServerHandler for JkServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "jk-tools".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some(
                "jk notebook tools: rag_search, read_note, edit_note, create_note, list_notes, list_tags, reindex, recent_journals, append_ai_journal".into(),
            ),
        }
    }

    tool_box!(@derive);
}

pub async fn run(notebook_dir: &str) {
    let server = JkServer::new(notebook_dir.to_string());
    let service = server
        .serve(stdio())
        .await
        .expect("MCP server failed to start");
    service.waiting().await.expect("MCP server error");
}
