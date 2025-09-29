use cloud_terrastodon_azure::prelude::fetch_all_groups;
use cloud_terrastodon_hcl::prelude::HCLImportBlock;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::eyre;
use itertools::Itertools;
use tracing::info;

pub async fn build_group_imports() -> Result<()> {
    info!("Fetching groups");
    let groups = fetch_all_groups()
        .await?
        .into_iter()
        .filter(|def| def.security_enabled)
        .collect_vec();

    let chosen = PickerTui::new(groups)
        .set_header("Groups to import")
        .pick_many()?;

    let imports: Vec<HCLImportBlock> = chosen.into_iter().map(|x| x.into()).collect_vec();

    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    HCLWriter::new(AppDir::Imports.join("group_imports.tf"))
        .overwrite(imports)
        .await?;

    Ok(())
}
