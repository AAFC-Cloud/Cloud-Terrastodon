use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraDirectoryAuditEvent;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// Read [Microsoft Entra directory audit events](https://learn.microsoft.com/en-us/graph/api/directoryaudit-list?view=graph-rest-1.0)
/// that target a group. Microsoft Graph applies the group filter before paging.
/// The selected account must have permission to read the tenant's audit log.
#[derive(Arbitrary, Debug, Facet)]
pub struct EntraGroupAuditLogListRequest {
    pub group_id: EntraGroupId,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraGroupAuditLogListRequest {}

impl HasCacheKey for EntraGroupAuditLogListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut key = CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "v1.0",
            "GET",
            "tenants",
            self.authentication.tenant_id.to_string().as_str(),
            "groups",
            self.group_id.to_string().as_str(),
            "directory-audit-log",
        ]))?;
        key.valid_for = Duration::from_secs(60);
        Ok(key)
    }
}

impl ExecutableRequest for EntraGroupAuditLogListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraDirectoryAuditEvent>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let mut url = Url::parse("https://graph.microsoft.com/v1.0/auditLogs/directoryAudits")?;
            // Microsoft documents the targetResources filter for group events:
            // https://learn.microsoft.com/en-us/powershell/module/microsoft.entra.reports/get-entraauditdirectorylog?view=entra-powershell
            url.query_pairs_mut()
                .append_pair(
                    "$filter",
                    &format!("targetResources/any(r:r/id eq '{}')", self.group_id),
                )
                .append_pair(
                    "$select",
                    "id,activityDateTime,activityDisplayName,category,result,loggedByService",
                )
                .append_pair("$orderby", "activityDateTime desc");
            MicrosoftGraphHelper::new(url, Some(self.cache_key()?), self.authentication)
                .fetch_all(&executor)
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupAuditLogListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupAuditLogListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    EntraGroupAuditLogListRequest => Vec<Arc<EntraDirectoryAuditEvent>>, effects = [Read]
);
