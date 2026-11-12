use crate::version::full_version;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_ui_egui::egui_main;
use facet::Facet;

/// Launch the egui-based graphical interface.
#[derive(Facet, Debug, Clone, Default)]
pub struct EguiArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Azure DevOps organization to browse alongside the selected Azure tenant.
    #[facet(figue::named, default)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
}

impl EguiArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> eyre::Result<()> {
        let authentication = self.authentication.resolve(executor).await?;
        egui_main(
            full_version()?.to_string(),
            self.org,
            authentication,
            executor,
        )
        .await?;
        Ok(())
    }
}
