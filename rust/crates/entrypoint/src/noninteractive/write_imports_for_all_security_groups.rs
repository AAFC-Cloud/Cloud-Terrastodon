use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::fetch_all_security_groups;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderBlock;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use tracing::info;

pub async fn write_imports_for_all_security_groups() -> Result<()> {
    info!("Fetching security groups");
    let security_groups = fetch_all_security_groups().await?;

    info!("Building import blocks");
    let mut imports: Vec<TofuImportBlock> = Vec::with_capacity(security_groups.len());
    for sg in security_groups {
        imports.push(sg.into())
    }

    info!("Writing import blocks");
    TofuWriter::new(AppDir::Imports.join("security_group_imports.tf"))
        .overwrite(imports)
        .await?
        .format()
        .await?;

    info!("Writing provider blocks");
    let providers = vec![TofuProviderBlock::AzureAD { alias: None }];
    TofuWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
