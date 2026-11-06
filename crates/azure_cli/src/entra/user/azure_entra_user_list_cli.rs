use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_azure::MicrosoftGraphProgress;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::borrow::Cow;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) users.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraUserListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        info!("Fetching users");
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        let users = EntraUserListRequest {
            auth_context: Cow::Borrowed(&tenant_auth_context),
            progress_hook: None,
        }
        .progress_hook(|progress: MicrosoftGraphProgress| {
            info!(
                total = ?progress.total,
                accumulated = progress.accumulated,
                received_this_page = progress.received_this_page,
                time_remaining = %humantime::format_duration(progress.time_remaining),
                "Fetched Entra users page"
            );
        })
        .execute(executor)
        .await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &users)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
