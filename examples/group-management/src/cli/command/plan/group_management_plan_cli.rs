use crate::DesireDocument;
#[cfg(test)]
use arbitrary::Arbitrary;
use cloud_terrastodon::app::CliOutput;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::facet;
use cloud_terrastodon::app::figue;
use cloud_terrastodon::auth_types::AuthSource;
use cloud_terrastodon::azure::AzureTenantId;
use cloud_terrastodon::execution::CloudTerrastodonExecutor;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use facet::Facet;
use std::path::PathBuf;

/// Compare desired relationships with fresh directory reads without applying changes.
#[derive(Facet)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct GroupManagementPlanArgs {
    /// JSON array describing existing groups, their tenants, and managed relationships.
    #[facet(figue::named, figue::label = "FILE")]
    pub desired: PathBuf,

    /// Also save the versioned JSON plan to this file.
    #[facet(figue::named, figue::label = "FILE")]
    pub out: Option<PathBuf>,

    /// Explicitly select one tenant from the document for a tenant-scoped pipeline job.
    /// Without this flag, authentication must cover every tenant in the document.
    #[facet(figue::named, figue::label = "TENANT_ID")]
    pub only_tenant: Option<AzureTenantId>,

    /// Account ID or alias in each selected tenant; omission requires configured defaults.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,

    /// Registered source override; auto uses each account's preferred mechanism.
    #[facet(figue::named, default)]
    pub auth_source: AuthSource,
}

impl GroupManagementPlanArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let document = DesireDocument::try_from_path(&self.desired)?;
        let account = self.account.as_ref().map(ToString::to_string);
        let plan = document
            .try_plan(
                self.only_tenant,
                account.as_deref(),
                self.auth_source,
                executor,
            )
            .await?;
        if let Some(path) = self.out {
            executor.cancellation().bail_if_cancelled()?;
            plan.save(&path, executor.cancellation())?;
        }
        let result = if plan.has_blockers() {
            Err(eyre::eyre!("Plan has blockers; no changes were made"))
        } else {
            Ok(())
        };
        Ok(CliOutput::facet(plan).with_result(result))
    }
}
