use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::PolicySetDefinition;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure policy set definitions interactively.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicySetDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePolicySetDefinitionBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure policy set definitions...");
        let policy_set_definitions = fetch_all_policy_set_definitions(tenant_id).await?;
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

        let chosen: Vec<PolicySetDefinition> = PickerTui::new()
            .set_header("Select Azure policy set definitions")
            .pick_many(choices)?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;

        Ok(())
    }
}
