use crate::portal_address::format_shell_command;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure_cli::AzureArgs;
use cloud_terrastodon_azure_cli::AzureCommand;
use cloud_terrastodon_azure_cli::resource_group::AzureResourceGroupArgs;
use cloud_terrastodon_azure_cli::resource_group::AzureResourceGroupCommand;
use cloud_terrastodon_azure_cli::resource_group::AzureResourceGroupListArgs;
use eyre::Result;

/// Display the CLI equivalent of the inventory request with its resolved identity.
pub(crate) fn resource_group_list_command(
    authentication: AzureAuthenticationArgs,
) -> Result<String> {
    let args = AzureArgs {
        command: AzureCommand::ResourceGroup(AzureResourceGroupArgs {
            command: AzureResourceGroupCommand::List(AzureResourceGroupListArgs { authentication }),
        }),
    };
    format_shell_command("ct az".into(), &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure::AzureTenantId;
    use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;

    #[test]
    fn command_round_trips_the_inventory_authentication_selection() -> Result<()> {
        let account: CloudTerrastodonAccountArgument = "personal".parse()?;
        let tenant_id: AzureTenantId = "00000000-0000-0000-0000-000000000002".parse()?;
        let auth_source = AzureBearerAuthSource::AzureCli;
        let command = resource_group_list_command(AzureAuthenticationArgs {
            account: Some(account.clone()),
            tenant: Some(tenant_id.into()),
            auth_source: auth_source.into(),
        })?;
        let args = command
            .strip_prefix("ct az ")
            .ok_or_else(|| eyre::eyre!("missing Azure command prefix"))?
            .split_whitespace()
            .collect::<Vec<_>>();
        let parsed: AzureArgs = figue::from_slice(&args)
            .into_result()
            .expect("valid CLI")
            .get();
        let AzureCommand::ResourceGroup(AzureResourceGroupArgs {
            command: AzureResourceGroupCommand::List(args),
        }) = parsed.command
        else {
            panic!("expected resource-group list");
        };
        assert_eq!(args.authentication.account, Some(account));
        assert_eq!(args.authentication.tenant, Some(tenant_id.into()));
        assert_eq!(args.authentication.auth_source, auth_source.into());
        Ok(())
    }
}
