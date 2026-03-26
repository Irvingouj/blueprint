use chrono::Utc;
use tracing::info;

use crate::error::{BlueprintError, Result};
use crate::model::Frontmatter;
use crate::{frontmatter, storage, validator};

pub async fn run(
    handle: &str,
    file: Option<&str>,
    content: Option<&str>,
    global: bool,
) -> Result<()> {
    // 1. Get the raw markdown
    let raw = match (file, content) {
        (Some(path), _) => {
            info!(path = %path, "reading blueprint from file");
            tokio::fs::read_to_string(path).await.map_err(|e| {
                BlueprintError::Other(format!("failed to read file '{path}': {e}"))
            })?
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
            references: Vec::new(),
            base_dir: None,
            saved_at: None,
        },
    };

    // 3. Inject metadata
    let cwd = std::env::current_dir()?;
    fm.base_dir = Some(cwd.display().to_string());
    fm.saved_at = Some(Utc::now());

    // 4. Validate
    let warnings = validator::validate_frontmatter(&fm)?;
    for w in &warnings {
        eprintln!("Warning: {}", w.message);
    }

    // 5. Recompose
    let document = frontmatter::compose(&fm, body)?;

    // 6. Write
    let dir = storage::storage_dir(global)?;
    storage::write_blueprint(&dir, handle, &document).await?;

    let path = dir.join(format!("{handle}.md"));
    println!("Saved blueprint '{handle}' to {}", path.display());
    Ok(())
}
