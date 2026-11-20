use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_cli::AzureArgs;
use cloud_terrastodon_azure_cli::AzureCommand;
use cloud_terrastodon_azure_cli::resource_group::AzureResourceGroupArgs;
use cloud_terrastodon_azure_cli::resource_group::AzureResourceGroupCommand;
use cloud_terrastodon_azure_cli::resource_group::AzureResourceGroupListArgs;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use eyre::Result;
use figue::ToArgs;

/// Display the CLI equivalent of the inventory request with its resolved identity.
pub(crate) fn resource_group_list_command(
    authentication: AzureBearerAuthenticationSelection,
) -> Result<String> {
    let args = AzureArgs {
        command: AzureCommand::ResourceGroup(AzureResourceGroupArgs {
            command: AzureResourceGroupCommand::List(AzureResourceGroupListArgs {
                authentication: AzureAuthenticationArgs {
                    tenant: authentication.tenant_id.into(),
                    account: Some(CloudTerrastodonAccountArgument::Id(
                        authentication.account_id,
                    )),
                    auth_source: authentication.auth_source.into(),
                },
            }),
        }),
    };
    Ok(format!(
        "ct az {}",
        args.to_args_string()?.to_string_lossy()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;

    #[test]
    fn command_round_trips_the_inventory_authentication_selection() -> Result<()> {
        let account_id = "00000000-0000-0000-0000-000000000001".parse()?;
        let tenant_id = "00000000-0000-0000-0000-000000000002".parse()?;
        let auth_source = AzureBearerAuthSource::AzureCli;
        let command = resource_group_list_command(AzureBearerAuthenticationSelection {
            account_id,
            tenant_id,
            auth_source,
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
        assert_eq!(
            args.authentication.account,
            Some(CloudTerrastodonAccountArgument::Id(account_id))
        );
        assert_eq!(
            args.authentication.tenant.to_string(),
            tenant_id.to_string()
        );
        assert_eq!(args.authentication.auth_source, auth_source.into());
        Ok(())
    }
}
