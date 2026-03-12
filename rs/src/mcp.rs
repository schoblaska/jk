use crate::{embed, search};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::Parameters,
    model::*,
    schemars, tool, tool_box,
    transport::io::stdio,
};
use std::path::Path;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchParams {
    /// Search query for semantic search over note embeddings
    query: String,
    /// Maximum number of results to return (default: 20)
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ReadNoteParams {
    /// Path to the note, relative to the notebook directory (e.g. "ai/a1b2.md")
    path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GrepParams {
    /// Regex pattern to search for in note contents
    pattern: String,
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

#[derive(Clone)]
pub struct JkServer {
    notebook_dir: String,
}

impl JkServer {
    fn new(notebook_dir: String) -> Self {
        Self { notebook_dir }
    }

    #[tool(description = "Semantic search over notes using embeddings. Returns TSV: score, file, line, heading, title")]
    async fn search_notes(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<SearchParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let query = params.query;
        let limit = params.limit.unwrap_or(20);
        tokio::task::spawn_blocking(move || {
            let results = search::search(&dir, &query)?;
            Ok(results
                .iter()
                .take(limit)
                .map(|r| {
                    format!(
                        "{:.3}\t{}\t{}\t{}\t{}",
                        r.sim, r.file, r.line, r.heading, r.title
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"))
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

    #[tool(description = "Read the full contents of a note by its relative path")]
    async fn read_note(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<ReadNoteParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let path = params.path;
        tokio::task::spawn_blocking(move || {
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
                return Err("Only .md files can be read".into());
            }
            std::fs::read_to_string(&canonical).map_err(|e| format!("Read error: {e}"))
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tool(description = "Search note contents using ripgrep (regex, smart-case)")]
    async fn grep_notes(
        &self,
        #[tool(aggr)] Parameters(params): Parameters<GrepParams>,
    ) -> Result<String, String> {
        let dir = self.notebook_dir.clone();
        let pattern = params.pattern;
        tokio::task::spawn_blocking(move || {
            let out = std::process::Command::new("rg")
                .args(["--no-heading", "--smart-case", &pattern])
                .current_dir(&dir)
                .output()
                .map_err(|e| format!("rg error: {e}"))?;
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
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

    tool_box!(JkServer {
        search_notes,
        list_notes,
        read_note,
        grep_notes,
        list_tags,
        create_note,
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

            // Dates
            let date_output = std::process::Command::new("date")
                .arg("+%a\t%b %d %Y\t%Y-%m-%d")
                .output()
                .map_err(|e| format!("date: {e}"))?;
            let date_str = String::from_utf8_lossy(&date_output.stdout);
            let parts: Vec<&str> = date_str.trim().split('\t').collect();
            if parts.len() != 3 {
                return Err("date parse error".into());
            }
            let pretty = format!("{}, {}", parts[0], parts[1]);
            let iso = parts[2];

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
                "jk notebook tools: search, list, read, grep, tag, and create notes".into(),
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
