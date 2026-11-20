use crate::version::full_version;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::WorkCacheActivity;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_config::EguiConfig;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_ui_egui::egui_main;
use cloud_terrastodon_ui_egui::egui_main_with_shared_cache;
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
        self.invoke_with_cache_activity(executor, log_capture, None)
            .await
    }

    pub(crate) async fn invoke_with_cache_activity(
        self,
        executor: &CloudTerrastodonExecutor,
        log_capture: LogBuffer,
        cache_activity: Option<WorkCacheActivity>,
    ) -> eyre::Result<()> {
        let account_store = CloudTerrastodonAccountStore::configured();
        // Revalidate the remembered registration when the CLI leaves account selection to the window.
        let authentication = if self.authentication.auth_source == AuthSource::None {
            Ok(None)
        } else if self.authentication.account.is_none()
            && self.authentication.tenant.is_none()
            && self.authentication.auth_source == AuthSource::Auto
            && let Some(saved) = EguiConfig::load().await?.azure_authentication
            && let Ok(selection) = account_store.resolve_azure_selection(
                saved.tenant_id,
                Some(&saved.account_id.to_string()),
                saved.auth_source.into(),
            )
        {
            Ok(Some(selection))
        } else {
            self.authentication.resolve(executor).await.map(Some)
        };
        let app_info = full_version()?.to_string();
        match cache_activity {
            Some(activity) => {
                egui_main_with_shared_cache(
                    app_info,
                    self.org,
                    authentication,
                    account_store,
                    log_capture,
                    executor,
                    activity,
                )
                .await
            }
            None => {
                egui_main(
                    app_info,
                    self.org,
                    authentication,
                    account_store,
                    log_capture,
                    executor,
                )
                .await
            }
        }
    }
}
