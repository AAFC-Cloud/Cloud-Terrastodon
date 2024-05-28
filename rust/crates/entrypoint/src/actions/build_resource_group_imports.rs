use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_resource_groups;
use fzf::pick_many;
use fzf::FzfArgs;
use itertools::Itertools;
use tofu::prelude::TofuImportWriter;
use tracing::info;

pub async fn build_resource_group_imports() -> Result<()> {
    info!("Fetching resource groups");
    let resource_groups = fetch_resource_groups().await?;

    let chosen = pick_many(FzfArgs {
        choices: resource_groups,
        prompt: Some("Groups to import: ".to_string()),
        header: None,
    })?;

    let imports = chosen.into_iter().map(|x| x.into()).collect_vec();

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    
    TofuImportWriter::new("resource_group_imports.tf")
        .overwrite(imports)
        .await?;

    Ok(())
}
