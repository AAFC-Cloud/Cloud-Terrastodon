use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use std::sync::Arc;
use tracing::info;

/// Arguments for browsing Azure resources interactively.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceBrowseArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        info!("Fetching Azure resources...");
        let resources = fetch_all_resources(&tenant_auth_context, executor).await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let choices = resources.into_iter().map(|resource| Choice {
            key: format!("{} - {}", resource.name, resource.id.expanded_form()),
            value: resource,
        });

        let (chosen, maybe_error): (Vec<Arc<Resource>>, _) = PickerTui::<_>::new()
            .set_header("Select Azure resources")
            .pick_many(choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &chosen)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;

        maybe_error
    }
}
