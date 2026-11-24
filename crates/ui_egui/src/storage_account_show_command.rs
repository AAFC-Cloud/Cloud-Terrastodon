use crate::portal_address::format_shell_command;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure_cli::AzureArgs;
use cloud_terrastodon_azure_cli::AzureCommand;
use cloud_terrastodon_azure_cli::storage::AzureStorageArgs;
use cloud_terrastodon_azure_cli::storage::AzureStorageCommand;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountArgs;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountCommand;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountShowArgs;
use eyre::Result;

/// Format the CLI equivalent of the typed get request with its resolved identity.
pub(crate) fn storage_account_show_command(
    authentication: AzureAuthenticationArgs,
    id: &AzureStorageAccountId,
    no_cache: bool,
    format: Option<OutputFormat>,
) -> Result<String> {
    let args = AzureArgs {
        command: AzureCommand::Storage(AzureStorageArgs {
            command: AzureStorageCommand::Account(AzureStorageAccountArgs {
                command: AzureStorageAccountCommand::Show(AzureStorageAccountShowArgs {
                    authentication,
                    storage_account: id.clone(),
                    no_cache,
                }),
            }),
        }),
    };
    let format_flag = match format {
        None => "",
        Some(OutputFormat::Json) => " --output-format json",
        Some(OutputFormat::FacetPretty) => " --output-format facet-pretty",
        Some(OutputFormat::Text) => " --output-format text",
        Some(OutputFormat::Auto) => " --output-format auto",
    };
    format_shell_command(format!("ct{format_flag} az"), &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_app::GlobalArgs;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure::AzureTenantId;
    use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
    use facet::Facet;

    #[derive(Facet)]
    struct Cli {
        #[facet(flatten)]
        global_args: GlobalArgs,
        #[facet(figue::subcommand)]
        command: Command,
    }

    #[derive(Facet)]
    #[repr(u8)]
    enum Command {
        #[facet(figue::alias = "az")]
        Azure(AzureArgs),
    }

    #[test]
    fn storage_show_command_preserves_resource_identity_and_output_format() -> Result<()> {
        let account: CloudTerrastodonAccountArgument = "personal".parse()?;
        let tenant_id: AzureTenantId = "00000000-0000-0000-0000-000000000002".parse()?;
        let auth_source = AzureBearerAuthSource::AzureCli;
        let authentication = AzureAuthenticationArgs {
            account: Some(account.clone()),
            tenant: Some(tenant_id.into()),
            auth_source: auth_source.into(),
        };
        let id = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000003",
            "/resourceGroups/synthetic(group)",
            "/providers/Microsoft.Storage/storageAccounts/syntheticstorage",
        )
        .parse()?;
        let command = storage_account_show_command(
            authentication,
            &id,
            true,
            Some(OutputFormat::FacetPretty),
        )?;
        assert!(command.contains("'/subscriptions/"));
        let tokens = command
            .split_whitespace()
            .skip(1)
            .map(|token| token.trim_matches('\''))
            .collect::<Vec<_>>();
        let cli: Cli = figue::from_slice(&tokens)
            .into_result()
            .expect("valid CLI")
            .get();
        let Command::Azure(AzureArgs {
            command:
                AzureCommand::Storage(AzureStorageArgs {
                    command:
                        AzureStorageCommand::Account(AzureStorageAccountArgs {
                            command: AzureStorageAccountCommand::Show(args),
                        }),
                }),
        }) = cli.command
        else {
            panic!("expected Azure storage account show");
        };
        assert_eq!(
            cli.global_args.output_format,
            Some(OutputFormat::FacetPretty)
        );
        assert_eq!(args.storage_account, id);
        assert_eq!(args.authentication.account, Some(account));
        assert_eq!(args.authentication.tenant, Some(tenant_id.into()));
        assert_eq!(args.authentication.auth_source, auth_source.into());
        assert!(args.no_cache);
        Ok(())
    }
}
