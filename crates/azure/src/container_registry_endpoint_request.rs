use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ContainerRegistryEndpoint;
use cloud_terrastodon_azure_types::ContainerRegistryId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use eyre::Result;
use eyre::ensure;
use std::borrow::Cow;
use std::path::PathBuf;

/// Resolve the service-assigned login server, including registries with DNS-label hashing.
/// [Registry loginServer](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(facet::Facet)]
pub struct ContainerRegistryEndpointRequest<'a> {
    pub registry_id: Cow<'a, ContainerRegistryId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
}
pub fn fetch_container_registry_endpoint<'a>(
    registry_id: &'a ContainerRegistryId,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> ContainerRegistryEndpointRequest<'a> {
    ContainerRegistryEndpointRequest {
        registry_id: Cow::Borrowed(registry_id),
        auth_context: Cow::Borrowed(auth_context),
        cancellation: cancellation.clone(),
    }
}
#[async_trait]
impl CacheableCommand for ContainerRegistryEndpointRequest<'_> {
    type Output = ContainerRegistryEndpoint;
    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            PathBuf::from("rest/container-registry/tenants")
                .join(self.auth_context.tenant_id.to_string())
                .join("subscriptions")
                .join(
                    self.registry_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                )
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    self.registry_id
                        .resource_group_id
                        .resource_group_name
                        .as_str(),
                ))
                .join("registries")
                .join(self.registry_id.container_registry_name.as_str())
                .join("endpoint"),
        )
    }
    async fn run(self) -> Result<Self::Output> {
        let query = format!(
            "Resources | where id =~ {} | project id, loginServer=tostring(properties.loginServer)",
            facet_json::to_string(&self.registry_id.expanded_form())?
        );
        let mut graph = ResourceGraphHelper::new(
            query,
            Some(self.cache_key()),
            &self.auth_context,
            &self.cancellation,
        );
        let mut endpoints: Vec<ContainerRegistryEndpoint> = graph.collect_all().await?;
        ensure!(
            endpoints.len() == 1,
            "expected one registry endpoint in the selected tenant, found {}",
            endpoints.len()
        );
        endpoints
            .pop()
            .ok_or_else(|| eyre::eyre!("registry endpoint was not returned"))
    }
}
cloud_terrastodon_command::impl_cacheable_into_future!(ContainerRegistryEndpointRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(ContainerRegistryEndpointRequest<'static>);

impl<'a> arbitrary::Arbitrary<'a> for ContainerRegistryEndpointRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            registry_id: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),

            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryEndpointRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(ContainerRegistryEndpointRequest<'static> => ContainerRegistryEndpoint, effects = [Read]);
