use super::jump_to_block::jump_to_block;
use anyhow::Result;
use pathing_types::IgnoreDir;
pub async fn list_imports() -> Result<()> {
    jump_to_block(IgnoreDir::Imports.into()).await
}
