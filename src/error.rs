use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlueprintError {
    #[error("Blueprint '{handle}' not found in {search_path}")]
    NotFound { handle: String, search_path: String },

    #[error("Invalid frontmatter: {0}")]
    FrontmatterParse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),

    #[error("Resolution failed for {reference}: {reason}")]
    ResolutionFailed { reference: String, reason: String },

    #[error("Resolution timed out for {reference} after {timeout_secs}s")]
    Timeout { reference: String, timeout_secs: u64 },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BlueprintError>;
