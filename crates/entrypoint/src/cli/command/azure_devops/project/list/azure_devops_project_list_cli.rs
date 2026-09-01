use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
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
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,

    /// Tenant id or tracked alias for delegated authentication.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureDevOpsProjectListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let azure_devops_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let projects =
            fetch_all_azure_devops_projects(&org_url, &azure_devops_auth_context).await?;
        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &projects)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(facet::Facet, Debug)]
    struct ParseArgs {
        #[facet(flatten)]
        args: AzureDevOpsProjectListArgs,
    }

    #[test]
    fn parses_an_explicit_tenant_alias() {
        let parsed: ParseArgs = figue::from_slice(&["--tenant", "agr"]).unwrap();
        assert_eq!(
            parsed.args.tenant.as_ref().map(ToString::to_string),
            Some("agr".to_owned())
        );
    }

    #[tokio::test]
    async fn project_list_authenticates_before_reading_organization_or_cache() {
        let args = AzureDevOpsProjectListArgs {
            org: Some("offline-test-organization".parse().unwrap()),
            tenant: None,
        };
        let error = args
            .invoke(&AuthContext::explicit(AuthSource::WorkloadIdentity))
            .await
            .expect_err("missing WIF configuration should fail before REST/cache work");
        assert!(
            error
                .to_string()
                .contains("workload identity authentication requires")
        );
    }

    #[tokio::test]
    async fn browser_project_list_requires_a_tenant_without_a_stored_session() {
        let args = AzureDevOpsProjectListArgs {
            org: Some("offline-test-organization".parse().unwrap()),
            tenant: None,
        };
        let error = args
            .invoke(&AuthContext::explicit(AuthSource::Browser))
            .await
            .expect_err("browser auth should fail locally without tenant/session context");
        assert!(error.to_string().contains("explicit tenant"));
    }

    #[tokio::test]
    async fn headless_cli_project_list_fails_before_cli_or_rest_access() {
        let auth_context = AuthContext::resolve(AuthSource::AzureCli)
            .expect("resolving an explicit CLI source should be local");
        if !auth_context.is_headless() {
            // The test is specifically about the non-interactive path. Cargo
            // tests are normally headless, but avoid making that assumption a
            // correctness requirement for an unusual interactive harness.
            return;
        }
        let args = AzureDevOpsProjectListArgs {
            org: Some("offline-test-organization".parse().unwrap()),
            tenant: None,
        };
        let error = args
            .invoke(&auth_context)
            .await
            .expect_err("headless CLI auth should fail before command or REST access");
        assert!(error.to_string().contains("disabled"));
    }
}
