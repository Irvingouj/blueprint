use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single reference entry in the frontmatter.
/// Supports both bare string sugar and explicit typed references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Reference {
    /// Bare string sugar: "src/main.rs" -> file reference
    Bare(String),
    /// Explicit typed reference
    Typed(TypedReference),
}

/// Explicit typed reference with a discriminator tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TypedReference {
    File { path: String },
    Url { url: String },
    GitDiff { path: String, range: String },
}

/// The YAML frontmatter of a blueprint document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub references: Vec<Reference>,
    pub base_dir: Option<String>,
    pub saved_at: Option<DateTime<Utc>>,
}

/// The output of resolving a single reference.
#[derive(Debug, Clone)]
pub struct ResolvedContent {
    /// Human-readable label (file path, URL, etc.)
    pub label: String,
    /// The resolved content (file contents, fetched HTML, diff output, etc.)
    pub content: String,
}

/// Context passed to each resolver during resolution.
pub struct ResolveContext {
    pub base_dir: PathBuf,
    pub timeout_override: Option<Duration>,
}
