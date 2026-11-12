use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraSecurityGroupListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclProviderBlock;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use tracing::info;

pub async fn write_imports_for_all_security_groups(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let cancellation = executor.cancellation();
    info!("Fetching security groups");
    let security_groups = EntraSecurityGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;

    info!("Building import blocks");
    let mut imports: Vec<HclImportBlock> = Vec::with_capacity(security_groups.len());
    for sg in security_groups.iter() {
        imports.push(sg.as_ref().into())
    }

    info!("Writing import blocks");
    HclWriter::new(AppDir::Imports.join("security_group_imports.tf"))
        .overwrite(imports, cancellation)
        .await?
        .format_file(cancellation)
        .await?;

    info!("Writing provider blocks");
    let providers = vec![HclProviderBlock::AzureAD { alias: None }];
    HclWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers, cancellation)
        .await?
        .format_file(cancellation)
        .await?;

    Ok(())
}
