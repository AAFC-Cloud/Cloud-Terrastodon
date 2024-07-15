use super::jump_to_block::jump_to_block;
use anyhow::Result;
use pathing::AppDir;
pub async fn list_imports() -> Result<()> {
    jump_to_block(AppDir::Imports.into()).await
}
