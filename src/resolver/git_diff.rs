use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use tracing::info;

use crate::error::{BlueprintError, Result};
use crate::model::{ResolveContext, ResolvedContent};
use crate::resolver::Resolver;

pub struct GitDiffResolver {
    path: String,
    range: String,
}

impl GitDiffResolver {
    pub fn new(path: String, range: String) -> Self {
        Self { path, range }
    }
}

#[async_trait]
impl Resolver for GitDiffResolver {
    async fn resolve(&self, ctx: &ResolveContext) -> Result<ResolvedContent> {
        info!(path = %self.path, range = %self.range, "resolving git-diff reference");

        let working_dir = if ctx.base_dir.exists() {
            ctx.base_dir.clone()
        } else {
            std::env::current_dir()?
        };

        let output = Command::new("git")
            .args(["diff", &self.range, "--", &self.path])
            .current_dir(&working_dir)
            .output()
            .await
            .map_err(|e| BlueprintError::ResolutionFailed {
                reference: format!("git-diff {}@{}", self.path, self.range),
                reason: format!("failed to run git: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BlueprintError::ResolutionFailed {
                reference: format!("git-diff {}@{}", self.path, self.range),
                reason: format!("git diff failed: {}", stderr.trim()),
            });
        }

        let diff = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(ResolvedContent {
            label: format!("{} ({})", self.path, self.range),
            content: diff,
        })
    }

    fn default_timeout(&self) -> Duration {
        Duration::from_secs(15)
    }
}
