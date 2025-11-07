use cloud_terrastodon_azure::prelude::fetch_all_security_groups;
use cloud_terrastodon_hcl::prelude::HclImportBlock;
use cloud_terrastodon_hcl::prelude::HclProviderBlock;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use tracing::info;

pub async fn write_imports_for_all_security_groups() -> Result<()> {
    info!("Fetching security groups");
    let security_groups = fetch_all_security_groups().await?;

    info!("Building import blocks");
    let mut imports: Vec<HclImportBlock> = Vec::with_capacity(security_groups.len());
    for sg in security_groups {
        imports.push(sg.into())
    }

    info!("Writing import blocks");
    HclWriter::new(AppDir::Imports.join("security_group_imports.tf"))
        .overwrite(imports)
        .await?
        .format_file()
        .await?;

    info!("Writing provider blocks");
    let providers = vec![HclProviderBlock::AzureAD { alias: None }];
    HclWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format_file()
        .await?;

    Ok(())
}
