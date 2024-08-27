use anyhow::anyhow;
use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::fetch_groups;
use cloud_terrastodon_core_fzf::pick_many;
use cloud_terrastodon_core_fzf::FzfArgs;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use itertools::Itertools;
use tracing::info;

pub async fn build_group_imports() -> Result<()> {
    info!("Fetching groups");
    let groups = fetch_groups()
        .await?
        .into_iter()
        .filter(|def| def.security_enabled)
        .collect_vec();

    let chosen = pick_many(FzfArgs {
        choices: groups,
        prompt: Some("Groups to import: ".to_string()),
        header: None,
    })?;

    let imports: Vec<TofuImportBlock> = chosen.into_iter().map(|x| x.into()).collect_vec();

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    TofuWriter::new(AppDir::Imports.join("group_imports.tf"))
        .overwrite(imports)
        .await?;

    Ok(())
}
