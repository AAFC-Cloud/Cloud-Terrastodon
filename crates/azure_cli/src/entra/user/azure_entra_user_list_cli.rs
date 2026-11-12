use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionEvent;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List Entra (Azure AD) users.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraUserListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraUserListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        info!("Fetching users");
        let authentication = self.authentication.resolve(executor).await?;
        let users = EntraUserListRequest { authentication }
            .execute(executor)
            .observe(|event| {
                if let ExecutionEvent::Progress(progress) = event {
                    info!(
                        total = ?progress.total,
                        accumulated = progress.accumulated,
                        received_this_page = progress.received_this_page,
                        time_remaining = %humantime::format_duration(progress.time_remaining),
                        "Fetched Entra users page"
                    );
                }
            })
            .await?;

        Ok(CliOutput::facet(users))
    }
}
