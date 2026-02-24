use crate::config::parse_ignore_patterns;
use std::path::Path;
use walkdir::WalkDir;

/// Recursively find all .md files, skipping .zk and ignored patterns.
/// Returns relative paths.
pub fn find_markdown_files(notebook_dir: &str) -> Vec<String> {
    let patterns = parse_ignore_patterns(notebook_dir);
    let base = Path::new(notebook_dir);
    let mut files = Vec::new();

    for entry in WalkDir::new(notebook_dir)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if e.file_type().is_dir() {
                if name == ".zk" {
                    return false;
                }
                if let Ok(rel) = e.path().strip_prefix(base) {
                    let rel_str = rel.to_string_lossy();
                    if should_skip(&rel_str, &patterns) {
                        return false;
                    }
                }
            }
            true
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if !name.ends_with(".md") || name == "index.md" {
            continue;
        }
        if let Ok(rel) = entry.path().strip_prefix(base) {
            files.push(rel.to_string_lossy().to_string());
        }
    }

    files
}

fn should_skip(rel: &str, patterns: &[String]) -> bool {
    for p in patterns {
        let pattern = p.trim_end_matches('/');
        if rel == pattern || rel.starts_with(&format!("{}/", pattern)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_logic() {
        let patterns = vec![".claude/".to_string(), "vendor/".to_string()];
        assert!(should_skip(".claude", &patterns));
        assert!(should_skip(".claude/foo", &patterns));
        assert!(should_skip("vendor", &patterns));
        assert!(!should_skip("src", &patterns));
        assert!(!should_skip("src/vendor.md", &patterns));
    }
}
