use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use tracing::info;

use crate::error::{BlueprintError, Result};
use crate::model::{ResolveContext, ResolvedContent};
use crate::resolver::Resolver;

pub struct FileResolver {
    path: String,
}

impl FileResolver {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Resolver for FileResolver {
    async fn resolve(&self, ctx: &ResolveContext) -> Result<ResolvedContent> {
        let file_path = if Path::new(&self.path).is_absolute() {
            PathBuf::from(&self.path)
        } else {
            ctx.base_dir.join(&self.path)
        };

        info!(path = %file_path.display(), "resolving file reference");

        let content = tokio::fs::read_to_string(&file_path).await.map_err(|e| {
            BlueprintError::ResolutionFailed {
                reference: self.path.clone(),
                reason: format!("failed to read '{}': {e}", file_path.display()),
            }
        })?;

        Ok(ResolvedContent {
            label: self.path.clone(),
            content,
        })
    }

    fn default_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}
