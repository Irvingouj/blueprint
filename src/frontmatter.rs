use crate::error::{BlueprintError, Result};
use crate::model::Frontmatter;

/// Split a markdown document into (frontmatter_yaml, body).
/// Returns None for frontmatter if the document has no frontmatter block.
pub fn split(document: &str) -> (Option<&str>, &str) {
    // Frontmatter must start at the very beginning of the document
    if !document.starts_with("---") {
        return (None, document);
    }

    // Skip the opening "---" line
    let after_open = &document[3..];
    let after_open = after_open
        .strip_prefix("\r\n")
        .or_else(|| after_open.strip_prefix('\n'))
        .unwrap_or(after_open);

    // Find the closing "---"
    if let Some(close_pos) = after_open.find("\n---") {
        let yaml = &after_open[..close_pos];
        let rest = &after_open[close_pos + 4..]; // skip "\n---"
        // Skip the newline after closing ---
        let body = rest
            .strip_prefix("\r\n")
            .or_else(|| rest.strip_prefix('\n'))
            .unwrap_or(rest);
        (Some(yaml), body)
    } else {
        // No closing ---, treat entire doc as body
        (None, document)
    }
}

/// Parse frontmatter YAML string into Frontmatter struct.
pub fn parse(yaml: &str) -> Result<Frontmatter> {
    serde_yaml_ng::from_str(yaml).map_err(|e| BlueprintError::FrontmatterParse(e.to_string()))
}

/// Serialize frontmatter and combine with body into a full document.
pub fn compose(frontmatter: &Frontmatter, body: &str) -> Result<String> {
    let yaml = serde_yaml_ng::to_string(frontmatter)?;
    Ok(format!("---\n{yaml}---\n{body}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_with_frontmatter() {
        let doc = "---\nfoo: bar\n---\n# Hello";
        let (yaml, body) = split(doc);
        assert_eq!(yaml, Some("foo: bar"));
        assert_eq!(body, "# Hello");
    }

    #[test]
    fn split_without_frontmatter() {
        let doc = "# Hello\nWorld";
        let (yaml, body) = split(doc);
        assert!(yaml.is_none());
        assert_eq!(body, "# Hello\nWorld");
    }

    #[test]
    fn split_no_closing_delimiter() {
        let doc = "---\nfoo: bar\n# No closing";
        let (yaml, body) = split(doc);
        assert!(yaml.is_none());
        assert_eq!(body, doc);
    }

    #[test]
    fn parse_frontmatter_with_references() {
        let yaml = "references:\n  - src/main.rs\n  - type: file\n    path: lib.rs\n";
        let fm = parse(yaml).unwrap();
        assert_eq!(fm.references.len(), 2);
    }

    #[test]
    fn round_trip() {
        let fm = Frontmatter {
            references: vec![],
            base_dir: Some("D:\\test".to_string()),
            saved_at: None,
        };
        let composed = compose(&fm, "# Body\n").unwrap();
        let (yaml, body) = split(&composed);
        assert!(yaml.is_some());
        let parsed = parse(yaml.unwrap()).unwrap();
        assert_eq!(parsed.base_dir, Some("D:\\test".to_string()));
        assert_eq!(body, "# Body\n");
    }
}
