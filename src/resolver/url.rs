use std::time::Duration;

use async_trait::async_trait;
use tracing::info;

use crate::error::{BlueprintError, Result};
use crate::model::{ResolveContext, ResolvedContent};
use crate::resolver::Resolver;

pub struct UrlResolver {
    url: String,
}

impl UrlResolver {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait]
impl Resolver for UrlResolver {
    async fn resolve(&self, _ctx: &ResolveContext) -> Result<ResolvedContent> {
        info!(url = %self.url, "resolving URL reference");

        let body = reqwest::get(&self.url)
            .await
            .map_err(|e| BlueprintError::ResolutionFailed {
                reference: self.url.clone(),
                reason: format!("HTTP request failed: {e}"),
            })?
            .text()
            .await
            .map_err(|e| BlueprintError::ResolutionFailed {
                reference: self.url.clone(),
                reason: format!("failed to read response body: {e}"),
            })?;

        Ok(ResolvedContent {
            label: self.url.clone(),
            content: body,
        })
    }

    fn default_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
