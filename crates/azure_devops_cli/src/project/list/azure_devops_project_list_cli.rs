use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// Azure DevOps project-related commands.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsProjectListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,

    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,
}

impl AzureDevOpsProjectListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let projects = AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(projects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_azure::AzureTenantArgument;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_execution::UnreachableAuthenticationProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn rejects_incompatible_cli_selection_before_configuration_or_provider_access()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(UnreachableAuthenticationProvider),
                    scope,
                );
                for (tenant, auth_source, expected) in [
                    (
                        Some(AzureTenantArgument::default()),
                        AuthSource::None,
                        "disabled",
                    ),
                    (
                        Some(AzureTenantArgument::default()),
                        AuthSource::PersonalAccessToken,
                        "cannot be combined",
                    ),
                    (None, AuthSource::Browser, "explicit tenant"),
                    (None, AuthSource::AzureCli, "explicit tenant"),
                    (None, AuthSource::WorkloadIdentity, "explicit tenant"),
                ] {
                    let error = AzureDevOpsProjectListArgs {
                        org: "synthetic".parse()?,
                        authentication: AzureDevOpsAuthenticationArgs {
                            tenant,
                            account: None,
                            auth_source,
                        },
                    }
                    .invoke(&executor)
                    .await
                    .expect_err("invalid selection must fail before execution");
                    assert!(error.to_string().contains(expected), "{error:#}");
                }
                eyre::Ok(())
            })
            .await
    }
}
