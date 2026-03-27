use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::PolicyDefinition;
use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure policy definitions interactively.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyDefinitionBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePolicyDefinitionBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure policy definitions...");
        let policy_definitions = fetch_all_policy_definitions(tenant_id).await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions",
        );

        let choices = policy_definitions.into_iter().map(|definition| Choice {
            key: match definition.description.as_ref() {
                Some(description) => format!("{definition} - {description}"),
                None => format!("{definition} - no description"),
            },
            value: definition,
        });

        let chosen: Vec<PolicyDefinition> = PickerTui::new()
            .set_header("Select Azure policy definitions")
            .pick_many(choices)?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;

        Ok(())
    }
}
