mod chunk;
mod config;
mod cosine;
mod db;
mod embed;
mod files;
mod mcp;
mod ollama;
mod search;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
        std::process::exit(1);
    }

    let cmd = &args[1];
    let notebook_dir = std::env::var("ZK_NOTEBOOK_DIR")
        .unwrap_or_else(|_| std::env::current_dir().unwrap().to_string_lossy().to_string());

    match cmd.as_str() {
        "search" => {
            let query = args[2..].join(" ");
            search::run(&notebook_dir, &query);
        }
        "mcp" => {
            if args.len() < 3 {
                eprintln!("Usage: jk-tools mcp <notebook-dir>");
                std::process::exit(1);
            }
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(mcp::run(&args[2]));
        }
        "embed" => {
            if args.len() > 2 {
                embed::incremental_reindex(&notebook_dir, &args[2..]);
            } else {
                embed::full_reindex(&notebook_dir);
            }
        }
        _ => {
            usage();
            std::process::exit(1);
        }
    }
}

fn usage() {
    eprintln!(
        "Usage: jk-tools <command> [args...]\n\n\
         Commands:\n  \
         embed [files...]     Full reindex (no args) or incremental (with files)\n  \
         search <query>       Semantic search, outputs TSV\n  \
         mcp <notebook-dir>   Start MCP server (stdio)"
    );
}
