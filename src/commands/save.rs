use chrono::Utc;
use tracing::info;

use crate::error::{BlueprintError, Result};
use crate::model::Frontmatter;
use crate::{frontmatter, storage, validator};

pub async fn run(
    handle: &str,
    description: Option<&str>,
    file: Option<&str>,
    content: Option<&str>,
    global: bool,
) -> Result<()> {
    // 1. Get the raw markdown
    let raw = match (file, content) {
        (Some(path), _) => {
            info!(path = %path, "reading blueprint from file");
            tokio::fs::read_to_string(path)
                .await
                .map_err(|e| BlueprintError::Other(format!("failed to read file '{path}': {e}")))?
        }
        (_, Some(text)) => text.to_string(),
        (None, None) => {
            return Err(BlueprintError::Other(
                "either --file or inline content must be provided".into(),
            ));
        }
    };

    // 2. Parse existing frontmatter or create a new one
    let (yaml_opt, body) = frontmatter::split(&raw);
    let mut fm = match yaml_opt {
        Some(yaml) => frontmatter::parse(yaml)?,
        None => Frontmatter {
            description: None,
            references: Vec::new(),
            base_dir: None,
            saved_at: None,
        },
    };

    // 3. Inject metadata
    let cwd = std::env::current_dir()?;
    fm.base_dir = Some(cwd.display().to_string());
    fm.saved_at = Some(Utc::now());

    // 4. Set description: CLI arg takes priority, then existing frontmatter, then auto-extract from H1
    fm.description = description
        .map(|s| s.to_string())
        .or(fm.description)
        .or_else(|| extract_h1(body));

    // 5. Validate
    let warnings = validator::validate_frontmatter(&fm)?;
    for w in &warnings {
        eprintln!("Warning: {}", w.message);
    }

    // 6. Recompose
    let document = frontmatter::compose(&fm, body)?;

    // 7. Write
    let dir = storage::storage_dir(global)?;
    storage::write_blueprint(&dir, handle, &document).await?;

    let path = dir.join(format!("{handle}.md"));
    println!("Saved blueprint '{handle}' to {}", path.display());
    Ok(())
}

/// Extract the first H1 heading from markdown body
fn extract_h1(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(content) = trimmed.strip_prefix("# ") {
            let desc = content.trim();
            if !desc.is_empty() {
                return Some(desc.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_h1_simple() {
        let body = "# Hello World\nSome content";
        assert_eq!(extract_h1(body), Some("Hello World".to_string()));
    }

    #[test]
    fn extract_h1_with_whitespace() {
        let body = "  #   Hello World  \nSome content";
        assert_eq!(extract_h1(body), Some("Hello World".to_string()));
    }

    #[test]
    fn extract_h1_empty_body() {
        let body = "";
        assert_eq!(extract_h1(body), None);
    }

    #[test]
    fn extract_h1_no_h1() {
        let body = "## Section\nSome content";
        assert_eq!(extract_h1(body), None);
    }

    #[test]
    fn extract_h1_empty_h1() {
        let body = "# \nSome content";
        assert_eq!(extract_h1(body), None);
    }

    #[test]
    fn extract_h1_with_hash_in_content() {
        let body = "# Fix #123: login bug\nSome content";
        assert_eq!(extract_h1(body), Some("Fix #123: login bug".to_string()));
    }

    #[test]
    fn extract_h1_multiline() {
        let body = "Some intro\n# The Real Title\nContent here";
        assert_eq!(extract_h1(body), Some("The Real Title".to_string()));
    }

    #[test]
    fn extract_h1_not_h2() {
        let body = "## Not this\n# But this\nContent";
        assert_eq!(extract_h1(body), Some("But this".to_string()));
    }
}
