use crate::azure_tenant_management_target::AzureTenantManagementTarget;
use crate::azure_tenant_management_update::AzureTenantManagementUpdate;
use crate::azure_tracked_tenant_catalogue::AzureTrackedTenantCatalogue;
use crate::gui_work_cache::GuiWorkCache;
use cloud_terrastodon_auth::read_azure_cli_local_accounts;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantDetailsRequest;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::add_tracked_tenant;
use cloud_terrastodon_azure::add_tracked_tenant_aliases;
use cloud_terrastodon_azure::discover_tracked_tenants_from_accounts;
use cloud_terrastodon_azure::forget_tracked_tenant;
use cloud_terrastodon_azure::get_tracked_tenant;
use cloud_terrastodon_azure::remove_tracked_tenant_aliases;
use cloud_terrastodon_azure::set_default_tenant_id;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::WrapErr;
use eyre::ensure;

/// A user action against the tracked tenant catalogue or a directory.
pub(crate) enum AzureTenantManagementAction {
    Reload,
    Track(AzureTenantId),
    Discover,
    SetDefault(AzureTenantId),
    Forget(AzureTenantId),
    AddAlias(AzureTenantId, AzureTenantAlias),
    RemoveAlias(AzureTenantId, AzureTenantAlias),
    Details {
        tenant: AzureTenantId,
        selected: Option<AzureBearerAuthenticationSelection>,
    },
}

impl AzureTenantManagementAction {
    pub const fn target(&self) -> AzureTenantManagementTarget {
        match self {
            Self::Details { .. } => AzureTenantManagementTarget::Directory,
            _ => AzureTenantManagementTarget::Catalogue,
        }
    }

    pub const fn description(&self) -> &'static str {
        match self {
            Self::Reload => "Reload tracked tenants",
            Self::Track(_) => "Track tenant",
            Self::Discover => "Discover tenants from Azure CLI",
            Self::SetDefault(_) => "Set default tenant",
            Self::Forget(_) => "Forget tracked tenant",
            Self::AddAlias(..) => "Add tenant alias",
            Self::RemoveAlias(..) => "Remove tenant alias",
            Self::Details { .. } => "Load tenant details",
        }
    }

    pub async fn run(
        self,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
    ) -> eyre::Result<AzureTenantManagementUpdate> {
        let cancellation = executor.cancellation();
        match self {
            Self::Reload => {}
            Self::Track(tenant) => {
                cancellation
                    .deferred("track Azure tenant", |_| add_tracked_tenant(tenant))
                    .await?;
            }
            Self::Discover => {
                let accounts = read_azure_cli_local_accounts(cancellation).await?;
                cancellation
                    .deferred("track discovered Azure tenants", |_| {
                        discover_tracked_tenants_from_accounts(accounts)
                    })
                    .await?;
            }
            Self::SetDefault(tenant) => {
                ensure!(
                    get_tracked_tenant(tenant).await?.is_some(),
                    "Tracked tenant '{tenant}' was not found."
                );
                set_default_tenant_id(tenant, cancellation).await?;
            }
            Self::Forget(tenant) => {
                let forgotten = cancellation
                    .deferred("forget Azure tenant", |_| forget_tracked_tenant(tenant))
                    .await?;
                ensure!(
                    forgotten.is_some(),
                    "Tracked tenant '{tenant}' was not found."
                );
            }
            Self::AddAlias(tenant, alias) => {
                cancellation
                    .deferred("add Azure tenant alias", async move |_| {
                        add_tracked_tenant_aliases(tenant, &[alias]).await
                    })
                    .await?;
            }
            Self::RemoveAlias(tenant, alias) => {
                cancellation
                    .deferred("remove Azure tenant alias", async move |_| {
                        remove_tracked_tenant_aliases(tenant, &[alias]).await
                    })
                    .await?;
            }
            Self::Details { tenant, selected } => {
                let authentication =
                    match selected.filter(|selection| selection.tenant_id == tenant) {
                        Some(selection) => selection,
                        None => executor
                            .resolve_azure_authentication(tenant, None, AuthSource::Auto)
                            .wrap_err(
                                "Select an account for this tenant or configure a default account.",
                            )?,
                    };
                let executor = work_cache.job_executor(executor)?;
                let details = AzureTenantDetailsRequest { authentication }
                    .execute(&executor)
                    .await?;
                return Ok(AzureTenantManagementUpdate::Details(details));
            }
        }
        let catalogue = AzureTrackedTenantCatalogue::load(cancellation).await?;
        Ok(AzureTenantManagementUpdate::Catalogue(catalogue))
    }
}
