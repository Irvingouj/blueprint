pub mod file;
pub mod git_diff;
pub mod url;

use std::time::Duration;

use async_trait::async_trait;
use futures::future::join_all;
use tokio::time::timeout;
use tracing::warn;

use crate::error::Result;
use crate::model::{Reference, ResolveContext, ResolvedContent, TypedReference};

use self::file::FileResolver;
use self::git_diff::GitDiffResolver;
use self::url::UrlResolver;

/// Async resolver trait — all resolvers must implement this.
#[async_trait]
pub trait Resolver: Send + Sync {
    /// Asynchronously resolve a reference into content.
    async fn resolve(&self, ctx: &ResolveContext) -> Result<ResolvedContent>;

    /// Default timeout duration for this resolver type.
    fn default_timeout(&self) -> Duration;
}

/// Map a Reference ADT variant to its concrete Resolver implementation.
fn to_resolver(reference: &Reference) -> Box<dyn Resolver> {
    match reference {
        Reference::Bare(path) => Box::new(FileResolver::new(path.clone())),
        Reference::Typed(typed) => match typed {
            TypedReference::File { path } => Box::new(FileResolver::new(path.clone())),
            TypedReference::Url { url } => Box::new(UrlResolver::new(url.clone())),
            TypedReference::GitDiff { path, range } => {
                Box::new(GitDiffResolver::new(path.clone(), range.clone()))
            }
        },
    }
}

/// Human-readable label for a reference (used in output headings).
pub fn format_label(reference: &Reference) -> String {
    match reference {
        Reference::Bare(p) => p.clone(),
        Reference::Typed(TypedReference::File { path }) => path.clone(),
        Reference::Typed(TypedReference::Url { url }) => url.clone(),
        Reference::Typed(TypedReference::GitDiff { path, range }) => {
            format!("{path} ({range})")
        }
    }
}

/// Resolve all references concurrently, each with its own timeout.
/// Never panics — failed/timed-out references produce `[unresolved: ...]` content.
pub async fn resolve_all(refs: &[Reference], ctx: &ResolveContext) -> Vec<ResolvedContent> {
    let futures: Vec<_> = refs
        .iter()
        .map(|r| {
            let resolver = to_resolver(r);
            let dur = ctx
                .timeout_override
                .unwrap_or_else(|| resolver.default_timeout());
            let label = format_label(r);
            async move {
                match timeout(dur, resolver.resolve(ctx)).await {
                    Ok(Ok(content)) => content,
                    Ok(Err(e)) => {
                        warn!(label = %label, error = %e, "resolution failed");
                        ResolvedContent {
                            label,
                            content: format!("[unresolved: {e}]"),
                        }
                    }
                    Err(_) => {
                        warn!(label = %label, timeout_secs = dur.as_secs(), "resolution timed out");
                        ResolvedContent {
                            label,
                            content: format!("[unresolved: timeout after {dur:?}]"),
                        }
                    }
                }
            }
        })
        .collect();

    join_all(futures).await
}
