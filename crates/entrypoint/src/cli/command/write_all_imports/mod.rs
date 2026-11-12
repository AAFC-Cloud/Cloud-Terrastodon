use crate::noninteractive::write_imports_for_all_resource_groups;
use crate::noninteractive::write_imports_for_all_role_assignments;
use crate::noninteractive::write_imports_for_all_security_groups;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Write Terraform import definitions for all supported resources.
#[derive(Facet, Debug, Clone, Default)]
pub struct WriteAllImportsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl WriteAllImportsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let authentication = self.authentication.resolve(executor).await?;
        write_imports_for_all_resource_groups(&authentication, executor).await?;
        write_imports_for_all_security_groups(&authentication, executor).await?;
        write_imports_for_all_role_assignments(&authentication, executor).await?;
        Ok(())
    }
}
