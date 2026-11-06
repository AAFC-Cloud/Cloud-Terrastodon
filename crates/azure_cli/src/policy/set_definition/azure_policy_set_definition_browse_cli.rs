use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::PolicySetDefinition;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use std::sync::Arc;
use tracing::info;

/// Arguments for browsing Azure policy set definitions interactively.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicySetDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePolicySetDefinitionBrowseArgs {
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
        info!("Fetching Azure policy set definitions...");
        let policy_set_definitions =
            fetch_all_policy_set_definitions(&tenant_auth_context, executor).await?;
        info!(
            count = policy_set_definitions.len(),
            "Fetched Azure policy set definitions",
        );

        let choices = policy_set_definitions.into_iter().map(|definition| Choice {
            key: match definition.description.as_ref() {
                Some(description) => format!("{definition} - {description}"),
                None => format!("{definition} - no description"),
            },
            value: definition,
        });

        let (chosen, maybe_error): (Vec<Arc<PolicySetDefinition>>, _) = PickerTui::<_>::new()
            .set_header("Select Azure policy set definitions")
            .pick_many(choices)
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
