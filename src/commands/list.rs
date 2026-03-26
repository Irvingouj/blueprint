use crate::error::Result;
use crate::storage;

pub async fn run(global: bool) -> Result<()> {
    let dir = storage::storage_dir(global)?;
    let handles = storage::list_handles(&dir).await?;

    if handles.is_empty() {
        let scope = if global { "global" } else { "project" };
        println!("No blueprints found ({scope} scope).");
        println!("Storage path: {}", dir.display());
        return Ok(());
    }

    for handle in &handles {
        println!("  {handle}");
    }

    Ok(())
}
