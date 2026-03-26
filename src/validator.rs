use url::Url;

use crate::error::{BlueprintError, Result};
use crate::model::{Frontmatter, Reference, TypedReference};

/// A non-fatal warning produced during validation.
#[derive(Debug)]
pub struct Warning {
    pub message: String,
}

/// Validate frontmatter structure before saving.
/// Returns warnings (non-fatal) on success, or an error (fatal) on failure.
pub fn validate_frontmatter(fm: &Frontmatter) -> Result<Vec<Warning>> {
    let mut warnings = Vec::new();

    for reference in &fm.references {
        match reference {
            Reference::Bare(path) => {
                validate_path_format(path)?;
            }
            Reference::Typed(typed) => match typed {
                TypedReference::File { path } => {
                    validate_path_format(path)?;
                }
                TypedReference::Url { url } => {
                    validate_url_format(url)?;
                }
                TypedReference::GitDiff { path, range } => {
                    validate_path_format(path)?;
                    validate_git_range_format(range)?;
                }
            },
        }
    }

    // Warn if no references (not an error, just informational)
    if fm.references.is_empty() {
        warnings.push(Warning {
            message: "Blueprint has no references — load will only output the plan body".into(),
        });
    }

    Ok(warnings)
}

fn validate_path_format(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(BlueprintError::Validation(
            "file path cannot be empty".into(),
        ));
    }
    // Check for obviously invalid characters (null byte)
    if path.contains('\0') {
        return Err(BlueprintError::Validation(format!(
            "file path contains invalid character: {path:?}"
        )));
    }
    Ok(())
}

fn validate_url_format(url_str: &str) -> Result<()> {
    if url_str.is_empty() {
        return Err(BlueprintError::Validation("URL cannot be empty".into()));
    }
    let parsed =
        Url::parse(url_str).map_err(|e| BlueprintError::Validation(format!("invalid URL '{url_str}': {e}")))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(BlueprintError::Validation(format!(
            "URL must use http or https scheme, got '{scheme}': {url_str}"
        )));
    }
    Ok(())
}

fn validate_git_range_format(range: &str) -> Result<()> {
    if range.is_empty() {
        return Err(BlueprintError::Validation(
            "git range cannot be empty".into(),
        ));
    }
    // Basic check: range should contain ".." for a proper range
    // or be a valid-looking ref (alphanumeric, ~, ^, etc.)
    if !range.contains("..") && !range.chars().all(|c| c.is_alphanumeric() || "~^-_/".contains(c))
    {
        return Err(BlueprintError::Validation(format!(
            "git range looks invalid: '{range}' (expected format like 'HEAD~3..HEAD')"
        )));
    }
    Ok(())
}
