use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::pick_oauth2_permission_grants;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;
use eyre::Result;
use std::io::Write;

/// Browse Entra OAuth2 permission grants interactively.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraOAuth2PermissionGrantBrowseArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &auth_context.auth_context.bind_executor(executor)?;
        let (chosen, maybe_error) = pick_oauth2_permission_grants(&auth_context, executor)
            .await
            .into_chosen_and_maybe_error()?;
        let chosen = chosen
            .into_iter()
            .map(|grant| grant.grant)
            .collect::<Vec<_>>();
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &chosen)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        maybe_error
    }
}
