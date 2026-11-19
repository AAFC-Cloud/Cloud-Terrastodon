use crate::version::full_version;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_ui_egui::egui_main;
use facet::Facet;
use teamy_tracing::capture::LogBuffer;

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
    pub async fn invoke(
        self,
        executor: &CloudTerrastodonExecutor,
        log_capture: LogBuffer,
    ) -> eyre::Result<()> {
        let account_store = CloudTerrastodonAccountStore::configured();
        let authentication = if self.authentication.auth_source == AuthSource::None {
            Ok(None)
        } else {
            self.authentication.resolve(executor).await.map(Some)
        };
        egui_main(
            full_version()?.to_string(),
            self.org,
            authentication,
            account_store,
            log_capture,
            executor,
        )
        .await?;
        Ok(())
    }
}
