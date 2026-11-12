use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Result;

/// Choose an organization PAT or an explicit tenant's Entra account before
/// resolving its non-secret metadata. Empty selectors use configured defaults
/// only within the chosen scope; resolution never probes another scope.
pub async fn prompt_azure_devops_authentication(
    organization: &AzureDevOpsOrganizationUrl,
    executor: &CloudTerrastodonExecutor,
) -> Result<AzureDevOpsAuthSelection> {
    let arguments = executor
        .cancellation()
        .cancellable_scope(async |_scope| {
            let tenant = prompt_line(
                "Entra tenant ID or alias (blank for organization PAT; 'default' for configured tenant): ",
            )
            .await?;
            let tenant = match tenant.trim() {
                "" => None,
                value => Some(value.parse()?),
            };
            let account = prompt_line("Account ID or alias (blank for this scope's configured default): ")
                .await?;
            let account = match account.trim() {
                "" => None,
                value => Some(value.parse()?),
            };
            let source = prompt_line("Authentication source (blank for account preference): ").await?;
            let auth_source = match source.trim() {
                "" => AuthSource::Auto,
                value => value.parse()?,
            };
            eyre::Ok(AzureDevOpsAuthenticationArgs {
                tenant,
                account,
                auth_source,
            })
        })
        .await?;
    arguments.resolve(organization, executor).await
}
