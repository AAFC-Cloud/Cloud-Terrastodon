use crate::AzureAuditFinding;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::collections::HashMap;
use std::sync::Arc;

/// Compare existing tag values with the nearest inventoried ancestor's tags.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureResourceTagAuditRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for AzureResourceTagAuditRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        AzureResourceListRequest {
            authentication: self.authentication,
        }
        .cache_keys_to_invalidate()
    }
}

impl AzureResourceTagAuditRequest {
    pub fn evaluate(
        resources: &[Arc<Resource>],
        cancellation: &CancellationScope,
    ) -> eyre::Result<Vec<AzureAuditFinding>> {
        let resources = resources
            .iter()
            .map(|resource| (resource.id.expanded_form(), resource))
            .collect::<HashMap<_, _>>();
        let mut findings = Vec::new();
        for (path, resource) in &resources {
            cancellation.bail_if_cancelled()?;
            let mut remaining: &str = path;
            while let Some((parent_path, _)) = remaining.rsplit_once('/') {
                if let Some(parent) = resources.get(parent_path) {
                    for (key, parent_value) in &parent.tags {
                        if let Some(value) = resource.tags.get(key)
                            && value != parent_value
                        {
                            findings.push(AzureAuditFinding::ResourceTagMismatch {
                                resource: resource.id.clone(),
                                parent: parent.id.clone(),
                                tag_key: key.clone(),
                                resource_tag_value: value.clone(),
                                parent_tag_value: parent_value.clone(),
                            });
                        }
                    }
                    break;
                }
                remaining = parent_path;
            }
        }
        Ok(findings)
    }
}

impl ExecutableRequest for AzureResourceTagAuditRequest {
    type Output = Vec<AzureAuditFinding>;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let resources = AzureResourceListRequest {
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            Self::evaluate(&resources, executor.cancellation())
        })
    }
}

cloud_terrastodon_registry_execution::register_executable_request!(AzureResourceTagAuditRequest => Vec<AzureAuditFinding>, effects = [Read]);

cloud_terrastodon_registry::register_thing!(AzureResourceTagAuditRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceTagAuditRequest);
