use std::path::PathBuf;

use tracing::info;

use crate::error::Result;
use crate::model::ResolveContext;
use crate::{frontmatter, resolver, storage};

pub async fn run(handle: &str, global: bool) -> Result<()> {
    let dir = storage::storage_dir(global)?;
    let raw = storage::read_blueprint(&dir, handle).await?;

    let (yaml_opt, body) = frontmatter::split(&raw);
    let fm = match yaml_opt {
        Some(yaml) => frontmatter::parse(yaml)?,
        None => {
            // No frontmatter, just print the body
            print!("{body}");
            return Ok(());
        }
    };

    let base_dir = fm
        .base_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    info!(
        handle = %handle,
        base_dir = %base_dir.display(),
        ref_count = fm.references.len(),
        "loading blueprint"
    );

    // Print the plan body
    print!("{body}");

    // If there are references, resolve and append
    if !fm.references.is_empty() {
        let ctx = ResolveContext {
            base_dir,
            timeout_override: None,
        };

        let resolved = resolver::resolve_all(&fm.references, &ctx).await;

        println!("\n---\n## Referenced Files\n");

        for content in &resolved {
            println!("### `{}`\n", content.label);
            let lang = detect_lang(&content.label);
            println!("```{lang}");
            println!("{}", content.content);
            println!("```\n");
        }
    }

    Ok(())
}

/// Best-effort language detection from file extension for code fences.
fn detect_lang(label: &str) -> &str {
    if label.starts_with("http://") || label.starts_with("https://") {
        return "html";
    }
    // git-diff references contain parentheses from range
    if label.contains("..") {
        return "diff";
    }
    match label.rsplit('.').next() {
        Some("rs") => "rust",
        Some("ts") => "typescript",
        Some("js") => "javascript",
        Some("py") => "python",
        Some("go") => "go",
        Some("toml") => "toml",
        Some("yaml" | "yml") => "yaml",
        Some("json") => "json",
        Some("md") => "markdown",
        Some("sh" | "bash") => "bash",
        Some("cs") => "csharp",
        Some("css") => "css",
        Some("html") => "html",
        Some("sql") => "sql",
        Some("diff") => "diff",
        Some("xml") => "xml",
        Some("java") => "java",
        Some("rb") => "ruby",
        Some("cpp" | "cc" | "cxx") => "cpp",
        Some("c" | "h") => "c",
        _ => "",
    }
}
