use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureNetworkSecurityGroup;
use cloud_terrastodon_azure_types::ResourceId;
use cloud_terrastodon_azure_types::ResourceType;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

const API_VERSION: &str = "2026-01-01";

/// Read one network security group and its rules through
/// [Network Security Groups - Get (2026-01-01)](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/networksecuritygroups/get?view=rest-virtualnetwork-2026-01-01).
/// Referenced resources are returned without requesting expansion.
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureNetworkSecurityGroupGetRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub network_security_group_id: ResourceId,
}

impl<'a> Arbitrary<'a> for AzureNetworkSecurityGroupGetRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: input.arbitrary()?,
            network_security_group_id: ResourceId {
                resource_group_id: input.arbitrary()?,
                resource_type: ResourceType::MICROSOFT_DOT_NETWORK_SLASH_NETWORKSECURITYGROUPS,
                resource_name: format!("nsg-{}", input.arbitrary::<u16>()?).into(),
            },
        })
    }
}

impl HasCacheKey for AzureNetworkSecurityGroupGetRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        ensure!(
            self.network_security_group_id.resource_type
                == ResourceType::MICROSOFT_DOT_NETWORK_SLASH_NETWORKSECURITYGROUPS,
            "expected an Azure network security group ID"
        );
        let group = &self.network_security_group_id.resource_group_id;
        CacheKey::from_relative_path(
            PathBuf::from("az/arm/tenant")
                .join(self.authentication.tenant_id.to_string())
                .join("subscriptions")
                .join(group.subscription_id.to_string())
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    &group.resource_group_name.as_str().to_ascii_lowercase(),
                ))
                .join("network-security-groups")
                .join(sanitize_windows_path_component(
                    &self
                        .network_security_group_id
                        .resource_name
                        .to_ascii_lowercase(),
                ))
                .join("get"),
        )
    }
}

impl InvalidatesOwnCacheKey for AzureNetworkSecurityGroupGetRequest {}

impl ExecutableRequest for AzureNetworkSecurityGroupGetRequest {
    type Error = eyre::Report;
    type Output = Arc<AzureNetworkSecurityGroup>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let key = self
                .cache_key()?
                .join(format!("rest/api-version/{API_VERSION}"))?;
            let group = &self.network_security_group_id.resource_group_id;
            let mut url = Url::parse("https://management.azure.com")?;
            url.path_segments_mut()
                .map_err(|_| eyre!("the ARM endpoint cannot contain path segments"))?
                .extend([
                    "subscriptions",
                    &group.subscription_id.to_string(),
                    "resourceGroups",
                    group.resource_group_name.as_str(),
                    "providers",
                    "Microsoft.Network",
                    "networkSecurityGroups",
                    self.network_security_group_id.resource_name.as_str(),
                ]);
            url.query_pairs_mut()
                .append_pair("api-version", API_VERSION);
            RestRequest::from_method_and_url(Method::GET, url)?
                .authentication(self.authentication.into())
                .cache(key)
                .receive_cached(&executor, CacheOperationOwner::of::<Self>())
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityGroupGetRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityGroupGetRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureNetworkSecurityGroupGetRequest => Arc<AzureNetworkSecurityGroup>,
    effects = [Read]
);
cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureNetworkSecurityGroupGetRequest,
    name = "network-security-group-get",
    cached
);
