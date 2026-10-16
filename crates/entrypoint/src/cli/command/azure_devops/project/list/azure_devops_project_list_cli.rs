use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
#[cfg(test)]
use cloud_terrastodon_credentials::AuthSource;
use eyre::Result;
use std::io::Write;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,

    /// Tenant id or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureDevOpsProjectListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let projects =
            fetch_all_azure_devops_projects(&self.org, &azure_devops_auth_context, cancellation)
                .await?;
        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &projects)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[derive(facet::Facet, Debug)]
    struct ParseArgs {
        #[facet(flatten)]
        args: AzureDevOpsProjectListArgs,
    }

    #[test]
    fn parses_an_explicit_tenant_alias() {
        let parsed: ParseArgs =
            figue::from_slice(&["--org", "offline-test-organization", "--tenant", "agr"]).unwrap();
        assert_eq!(
            parsed.args.tenant.as_ref().map(ToString::to_string),
            Some("agr".to_owned())
        );
    }

    #[tokio::test]
    async fn project_list_authenticates_before_reading_organization_or_cache() {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let cancellation = &cancellation;

                let args = AzureDevOpsProjectListArgs {
                    org: "offline-test-organization".parse().unwrap(),
                    tenant: None,
                };
                let error = args
                    .invoke(
                        &AuthContext::explicit(AuthSource::WorkloadIdentity),
                        cancellation,
                    )
                    .await
                    .expect_err("missing WIF configuration should fail before REST/cache work");
                assert!(
                    error
                        .to_string()
                        .contains("workload identity authentication requires")
                );

                Ok::<_, eyre::Report>(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn browser_project_list_requires_a_tenant_without_a_stored_session() {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let cancellation = &cancellation;

                let args = AzureDevOpsProjectListArgs {
                    org: "offline-test-organization".parse().unwrap(),
                    tenant: None,
                };
                let error = args
                    .invoke(&AuthContext::explicit(AuthSource::Browser), cancellation)
                    .await
                    .expect_err("browser auth should fail locally without tenant/session context");
                assert!(error.to_string().contains("explicit tenant"));

                Ok::<_, eyre::Report>(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn headless_cli_project_list_fails_before_cli_or_rest_access() {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let cancellation = &cancellation;

                let mut auth_context = AuthContext::explicit(AuthSource::AzureCli);
                if let AuthContext::Resolved { headless, .. } = &mut auth_context {
                    *headless = true;
                }
                // Cached project pages intentionally bypass fresh authentication. Use
                // a new cache namespace, without touching any existing user cache.
                let organization = format!("offline-test-{}", uuid::Uuid::new_v4());
                let args = AzureDevOpsProjectListArgs {
                    org: organization.parse().unwrap(),
                    tenant: None,
                };
                let error = args
                    .invoke(&auth_context, cancellation)
                    .await
                    .expect_err("headless CLI auth should fail before command or REST access");
                assert!(
                    error
                        .chain()
                        .any(|cause| cause.to_string().contains("disabled")),
                    "{error:#}"
                );

                Ok::<_, eyre::Report>(())
            })
            .await
            .unwrap();
    }
}
