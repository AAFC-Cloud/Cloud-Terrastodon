use super::jump_to_block::jump_to_block;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
pub async fn list_imports() -> Result<()> {
    jump_to_block(AppDir::Imports.into()).await
}
