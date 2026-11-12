use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraGroupListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::eyre;
use itertools::Itertools;
use tracing::info;

pub async fn build_group_imports(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let cancellation = executor.cancellation();
    info!("Fetching groups");
    let groups = EntraGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?
    .iter()
    .filter(|def| def.security_enabled)
    .cloned()
    .collect_vec();

    let chosen = PickerTui::<_>::new()
        .set_header("Groups to import")
        .pick_many(groups, executor.cancellation())
        .await?;

    let imports: Vec<HclImportBlock> = chosen
        .iter()
        .map(|group| group.as_ref().into())
        .collect_vec();

    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    HclWriter::new(AppDir::Imports.join("group_imports.tf"))
        .overwrite(imports, cancellation)
        .await?;

    Ok(())
}
