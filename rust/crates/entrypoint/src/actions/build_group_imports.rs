use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_groups;
use fzf::pick_many;
use fzf::FzfArgs;
use itertools::Itertools;
use tofu::prelude::TofuImportWriter;
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

    let imports = chosen.into_iter().map(|x| x.into()).collect_vec();

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }
    
    TofuImportWriter::new("group_imports.tf")
        .overwrite(imports)
        .await?;

    Ok(())
}
