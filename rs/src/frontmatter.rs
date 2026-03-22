use std::path::Path;

#[allow(dead_code)]
pub struct Frontmatter {
    pub title: String,
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

/// Parse frontmatter from the first ~10 lines of a markdown note.
pub fn parse_frontmatter(content: &str) -> Frontmatter {
    let mut title = String::new();
    let mut date = None;
    let mut tags = Vec::new();
    let mut description = None;

    for (i, line) in content.lines().take(10).enumerate() {
        if i == 0 {
            if let Some(t) = line.strip_prefix("# ") {
                title = t.trim().to_string();
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("date:") {
            // Extract ISO date from link target: [pretty](YYYY-MM-DD) or [pretty](../YYYY-MM-DD)
            if let Some(start) = rest.rfind('(') {
                if let Some(end) = rest.rfind(')') {
                    let target = &rest[start + 1..end];
                    // Strip leading ../ segments
                    let target = target.trim_start_matches("../");
                    // Validate looks like ISO date
                    if target.len() == 10 && target.as_bytes().get(4) == Some(&b'-') {
                        date = Some(target.to_string());
                    }
                }
            }
        } else if let Some(rest) = line.strip_prefix("tags:") {
            tags = rest
                .split_whitespace()
                .filter_map(|t| t.strip_prefix('#'))
                .filter(|t| !t.is_empty())
                .map(|t| t.trim_end_matches(',').to_string())
                .collect();
        } else if let Some(rest) = line.strip_prefix("description:") {
            let d = rest.trim();
            if !d.is_empty() {
                description = Some(d.to_string());
            }
        }
    }

    Frontmatter {
        title,
        date,
        tags,
        description,
    }
}

/// Extract markdown links from content, returning normalized relative paths.
/// `file_dir` is the directory of the source file (e.g. "ai" for "ai/a1b2.md", "" for root).
pub fn extract_links(content: &str, file_dir: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Scan for [text](target) patterns
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find ](
        if bytes[i] == b']' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            let start = i + 2;
            if let Some(end) = content[start..].find(')') {
                let target = &content[start..start + end];
                i = start + end + 1;

                // Skip URLs, anchors, empty
                if target.is_empty()
                    || target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with('#')
                {
                    continue;
                }

                // Strip anchor fragment
                let target = target.split('#').next().unwrap_or(target);
                if target.is_empty() {
                    continue;
                }

                // Resolve relative to file's directory
                let resolved = if file_dir.is_empty() {
                    Path::new(target)
                        .to_string_lossy()
                        .to_string()
                } else {
                    let joined = Path::new(file_dir).join(target);
                    normalize_path(&joined.to_string_lossy())
                };

                // Ensure .md extension
                let resolved = if resolved.ends_with(".md") {
                    resolved
                } else {
                    format!("{resolved}.md")
                };

                if seen.insert(resolved.clone()) {
                    links.push(resolved);
                }
            } else {
                i += 2;
            }
        } else {
            i += 1;
        }
    }

    links
}

/// Normalize path segments (resolve .. components).
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Check if a filename looks like a journal entry (YYYY-MM-DD.md in root).
pub fn is_journal(rel_path: &str) -> bool {
    // Must be in root directory (no /)
    if rel_path.contains('/') {
        return false;
    }
    let name = rel_path.strip_suffix(".md").unwrap_or(rel_path);
    if name.len() != 10 {
        return false;
    }
    let bytes = name.as_bytes();
    // YYYY-MM-DD
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_frontmatter() {
        let content = "# My Note\ndate: [Mon, Feb 23 2026](2026-02-23)\ndescription: A test note\ntags: #travel #italy #food\n\n## Body\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.title, "My Note");
        assert_eq!(fm.date.as_deref(), Some("2026-02-23"));
        assert_eq!(fm.tags, vec!["travel", "italy", "food"]);
        assert_eq!(fm.description.as_deref(), Some("A test note"));
    }

    #[test]
    fn parse_ai_note_date() {
        let content = "# AI Note\ndate: [Mon, Feb 23 2026](../2026-02-23)\ntags: #ai-generated\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.date.as_deref(), Some("2026-02-23"));
    }

    #[test]
    fn parse_no_frontmatter() {
        let content = "Just some text\nwith no headings\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.title, "");
        assert!(fm.date.is_none());
        assert!(fm.tags.is_empty());
    }

    #[test]
    fn parse_tags_with_commas() {
        let content = "# Note\ntags: #one, #two, #three\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.tags, vec!["one", "two", "three"]);
    }

    #[test]
    fn extract_links_basic() {
        let content = "See [My Note](a1b2) and [Other](../c3d4.md) for details.\n";
        let links = extract_links(content, "ai");
        assert_eq!(links, vec!["ai/a1b2.md", "c3d4.md"]);
    }

    #[test]
    fn extract_links_from_root() {
        let content = "Link to [note](a1b2) and [ai note](ai/x1y2)\n";
        let links = extract_links(content, "");
        assert_eq!(links, vec!["a1b2.md", "ai/x1y2.md"]);
    }

    #[test]
    fn extract_links_skips_urls() {
        let content = "[Google](https://google.com) and [note](a1b2)\n";
        let links = extract_links(content, "");
        assert_eq!(links, vec!["a1b2.md"]);
    }

    #[test]
    fn extract_links_deduplicates() {
        let content = "[A](note) and [B](note)\n";
        let links = extract_links(content, "");
        assert_eq!(links, vec!["note.md"]);
    }

    #[test]
    fn extract_links_strips_anchor() {
        let content = "[Section](note#heading)\n";
        let links = extract_links(content, "");
        assert_eq!(links, vec!["note.md"]);
    }

    #[test]
    fn journal_detection() {
        assert!(is_journal("2026-02-23.md"));
        assert!(is_journal("2025-01-01.md"));
        assert!(!is_journal("ai/2026-02-23.md"));
        assert!(!is_journal("notes.md"));
        // Note: 2026-13-01.md matches the pattern — no month/day range validation (fine for our use case)
        assert!(!is_journal("abcd-ef-gh.md"));
    }

    #[test]
    fn normalize_path_resolves_dotdot() {
        assert_eq!(normalize_path("ai/../foo"), "foo");
        assert_eq!(normalize_path("a/b/../c"), "a/c");
        assert_eq!(normalize_path("../foo"), "foo"); // can't go above root
    }
}
