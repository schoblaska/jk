pub struct Chunk {
    pub heading: String,
    pub lnum: usize,
    pub text: String,
}

/// Split markdown content into chunks by heading.
pub fn chunk_markdown(content: &str) -> Vec<Chunk> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut chunks = Vec::new();
    let mut title: Option<&str> = None;
    let mut heading: Option<&str> = None;
    let mut lnum: usize = 0;
    let mut text = String::new();

    for (i, line) in lines.iter().enumerate() {
        if i == 0 && line.starts_with("# ") {
            title = Some(line.trim_start_matches("# ").trim_start());
        }
        if is_heading(line) {
            // flush previous chunk
            if heading.is_some() {
                chunks.push(Chunk {
                    heading: heading.unwrap().to_string(),
                    lnum,
                    text: text.trim_end_matches('\n').to_string(),
                });
                text.clear();
            }
            heading = Some(line);
            lnum = i + 1; // 1-based
            let h = strip_heading_prefix(line);
            if let Some(t) = title {
                if h != t {
                    text.push_str(t);
                    text.push_str(" > ");
                }
            }
            text.push_str(line);
            text.push('\n');
        } else if heading.is_some() {
            text.push_str(line);
            text.push('\n');
        }
    }

    // flush last chunk
    if heading.is_some() {
        chunks.push(Chunk {
            heading: heading.unwrap().to_string(),
            lnum,
            text: text.trim_end_matches('\n').to_string(),
        });
    }

    chunks
}

fn is_heading(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() && bytes[i] == b'#' {
        i += 1;
    }
    i > 0 && i < bytes.len() && bytes[i] == b' '
}

fn strip_heading_prefix(line: &str) -> &str {
    let s = line.trim_start_matches('#');
    s.trim_start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_headings() {
        let chunks = chunk_markdown("just some text\nno headings here");
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn chunks_by_headings() {
        let md = "# Title\nsome intro\n\n## Section A\ncontent a\n\n## Section B\ncontent b\n";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].heading, "# Title");
        assert_eq!(chunks[0].lnum, 1);
        assert_eq!(chunks[0].text, "# Title\nsome intro");
        assert_eq!(chunks[1].heading, "## Section A");
        assert_eq!(chunks[1].lnum, 4);
        assert_eq!(chunks[1].text, "Title > ## Section A\ncontent a");
        assert_eq!(chunks[2].heading, "## Section B");
        assert_eq!(chunks[2].lnum, 7);
        assert_eq!(chunks[2].text, "Title > ## Section B\ncontent b");
    }

    #[test]
    fn no_prefix_when_heading_matches_title() {
        let md = "# Only Heading\nsome text\n";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks[0].text, "# Only Heading\nsome text");
    }

    #[test]
    fn handles_h3_headings() {
        let md = "# Top\n## Mid\n### Deep\ndeep content\n";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[2].heading, "### Deep");
    }
}
