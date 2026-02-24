use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default)]
struct Config {
    #[serde(default)]
    note: NoteConfig,
}

#[derive(Deserialize, Default)]
struct NoteConfig {
    #[serde(default)]
    ignore: Vec<String>,
}

/// Parse ignore patterns from .zk/config.toml.
pub fn parse_ignore_patterns(notebook_dir: &str) -> Vec<String> {
    let path = Path::new(notebook_dir).join(".zk").join("config.toml");
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    match basic_toml::from_str::<Config>(&content) {
        Ok(config) => config.note.ignore,
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(content: &str) -> Vec<String> {
        basic_toml::from_str::<Config>(content)
            .map(|c| c.note.ignore)
            .unwrap_or_default()
    }

    #[test]
    fn parses_ignore_list() {
        let content = r#"
[note]
template = "default.md"
ignore = [".claude/", ".submodules/", "vendor/"]
"#;
        assert_eq!(parse(content), vec![".claude/", ".submodules/", "vendor/"]);
    }

    #[test]
    fn empty_when_no_ignore() {
        let content = r#"
[note]
template = "default.md"
"#;
        assert!(parse(content).is_empty());
    }

    #[test]
    fn empty_for_empty_content() {
        assert!(parse("").is_empty());
    }
}
