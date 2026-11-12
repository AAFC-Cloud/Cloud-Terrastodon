use crate::AzureAppServiceListRequest;
use crate::AzureApplicationGatewayListRequest;
use crate::AzureCognitiveServicesAccountDeploymentListRequest;
use crate::AzureCognitiveServicesAccountListRequest;
use crate::AzureContainerInstanceListRequest;
use crate::AzureKeyVaultListRequest;
use crate::AzureNetworkInterfaceListRequest;
use crate::AzurePrivateEndpointListRequest;
use crate::AzurePublicIpListRequest;
use crate::AzureResourceGroupListRequest;
use crate::AzureResourceListRequest;
use crate::AzureResourceTagsGetRequest;
use crate::AzureRouteTableListRequest;
use crate::AzureStorageAccountListRequest;
use crate::AzureSubscriptionListRequest;
use crate::AzureVirtualMachineListRequest;
use crate::AzureVirtualNetworkListRequest;
use crate::azure_resource_tags_update_response::AzureResourceTagsUpdateResponse;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResourceId;
use cloud_terrastodon_azure_types::AzureResourceTags;
use cloud_terrastodon_azure_types::AzureResourceTagsPatch;
use cloud_terrastodon_azure_types::AzureResourceTagsPatchOperation;
use cloud_terrastodon_azure_types::AzureResourceTagsProperties;
use cloud_terrastodon_azure_types::ResourceTagsId;
use cloud_terrastodon_azure_types::ResourceType;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use url::Url;

/// Replace, merge, or selectively delete tags on one ARM resource or subscription.
/// [Tags Update At Scope](https://learn.microsoft.com/rest/api/resources/tags/update-at-scope?view=rest-resources-2021-04-01).
/// Uses API 2022-09-01 and retains mutation artifacts. Completion invalidates the
/// selected tag read and affected aggregate inventories. Accepted operations are
/// polled for at most 120 attempts within a ten-minute network/wait budget.
/// Credential acquisition and required persistence are outside that budget.
/// Timeout reports unconfirmed completion and invalidates affected reads because
/// the remote write may finish.
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureResourceTagsUpdateRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub id: Cow<'a, ResourceTagsId>,
    pub operation: AzureResourceTagsPatchOperation,
    pub tags: HashMap<String, String>,
}
impl<'a> Arbitrary<'a> for AzureResourceTagsUpdateRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            id: Cow::Owned(Arbitrary::arbitrary(u)?),
            operation: Arbitrary::arbitrary(u)?,
            tags: Arbitrary::arbitrary(u)?,
        })
    }
}

impl HasCacheKey for AzureResourceTagsUpdateRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from("rest/arm/tenants")
            .join(self.authentication.tenant_id.to_string())
            .join("resource-tags");
        for component in self
            .id
            .expanded_form()
            .to_ascii_lowercase()
            .trim_start_matches('/')
            .split('/')
        {
            path.push(sanitize_windows_path_component(component));
        }
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(path.join("update/api-version/2022-09-01"))?,
            valid_for: Duration::ZERO,
        })
    }
}
impl InvalidatesOwnCacheKey for AzureResourceTagsUpdateRequest<'_> {}
impl ExecutableRequest for AzureResourceTagsUpdateRequest<'_> {
    type Output = AzureResourceTags;
    type Error = eyre::Report;
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
            let key = self.cache_key()?;
            let tagged_scope = self.id.tagged_scope()?;
            let mut url = Url::parse("https://management.azure.com")?;
            url.set_path(&self.id.expanded_form().to_ascii_lowercase());
            url.query_pairs_mut()
                .append_pair("api-version", "2022-09-01");
            let body = facet_json::to_string(&AzureResourceTagsPatch {
                operation: self.operation,
                properties: AzureResourceTagsProperties { tags: self.tags },
            })?;
            executor.deferred("updating resource tags and invalidating affected reads", async |child| {
                let execution = async {
                    let started = Instant::now();
                    let limit = Duration::from_secs(10 * 60);
                    let mut response = RestRequest::from_method_and_url(Method::PATCH, url.clone())?
                        .authentication(self.authentication.into())
                        .body(body)
                        .cache(key.clone())
                        .transport_timeout(limit)
                        .receive_raw_with_decoder(|response| {
                            AzureResourceTagsUpdateResponse::from_response(response, &self.id, &url, None)
                        }, &child).await?;
                    for poll in 0..=120 {
                        match response {
                            AzureResourceTagsUpdateResponse::Complete(tags) => return Ok(tags),
                            AzureResourceTagsUpdateResponse::CompleteWithoutBody => {
                                let remaining = limit.saturating_sub(started.elapsed());
                                eyre::ensure!(!remaining.is_zero(), "Tags update completed but final-response read exceeded the network/wait budget");
                                return RestRequest::from_method_and_url(Method::GET, url.clone())?
                                    .authentication(self.authentication.into())
                                    .cache(CacheKey { scope: key.scope.join("final-response")?, valid_for: Duration::ZERO })
                                    .transport_timeout(remaining)
                                    .receive_with_validator(|response: AzureResourceTags| {
                                        response.validate_identity(&self.id)?;
                                        Ok(response)
                                    }, &child).await;
                            }
                            AzureResourceTagsUpdateResponse::Pending { location, retry_after } => {
                                eyre::ensure!(poll < 120 && retry_after < limit.saturating_sub(started.elapsed()),
                                    "Tags update completion is unconfirmed after bounded polling; the remote write may still finish");
                                tokio::time::sleep(retry_after).await;
                                let remaining = limit.saturating_sub(started.elapsed());
                                eyre::ensure!(!remaining.is_zero(), "Tags update completion is unconfirmed after the network/wait budget; the remote write may still finish");
                                response = RestRequest::from_method_and_url(Method::GET, location.clone())?
                                    .authentication(self.authentication.into())
                                    .cache(CacheKey { scope: key.scope.join("polls")?.join(poll.to_string())?, valid_for: Duration::ZERO })
                                    .transport_timeout(remaining)
                                    .receive_raw_with_decoder(|response| {
                                        AzureResourceTagsUpdateResponse::from_response(response, &self.id, &url, Some(&location))
                                    }, &child).await?;
                            }
                        }
                    }
                    eyre::bail!("Tags update completion is unconfirmed after bounded polling");
                }.await;
                // Any response/decode/artifact error can follow a completed remote
                // mutation. Attempt every invalidation before returning that error.
                // Inventories are currently cached per tenant. Invalidate only
                // the actual resource kind's inventory, plus the all-resources view.
                let inventory = match &tagged_scope {
                    ScopeImpl::ResourceGroup(_) => AzureResourceGroupListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::Subscription(_) => AzureSubscriptionListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::StorageAccount(_) => AzureStorageAccountListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::KeyVault(_) => AzureKeyVaultListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::VirtualNetwork(_) => AzureVirtualNetworkListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::RouteTable(_) => AzureRouteTableListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzureAppServiceResource(_) => AzureAppServiceListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzureCognitiveServicesAccountResource(_) => AzureCognitiveServicesAccountListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzureApplicationGatewayResource(_) => AzureApplicationGatewayListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzureContainerInstanceResource(_) => AzureContainerInstanceListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzureNetworkInterfaceResource(_) => AzureNetworkInterfaceListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzurePrivateEndpointResource(_) => AzurePrivateEndpointListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::AzurePublicIpResource(_) => AzurePublicIpListRequest {
                        authentication: self.authentication,
                    }.execute(&child).invalidate().await,
                    ScopeImpl::Resource(resource)
                        if resource.resource_type == ResourceType::MICROSOFT_DOT_COMPUTE_SLASH_VIRTUALMACHINES =>
                    {
                        AzureVirtualMachineListRequest {
                            authentication: self.authentication,
                        }.execute(&child).invalidate().await
                    }
                    _ => Ok(()),
                };
                let expanded = tagged_scope.expanded_form();
                let mut parts = expanded.rsplitn(3, '/');
                let _deployment_name = parts.next();
                let deployment_kind = parts.next();
                let deployment_parent = parts.next();
                let deployment_inventory = match (deployment_kind, deployment_parent) {
                    (Some(kind), Some(parent)) if kind.eq_ignore_ascii_case("deployments") => {
                        if let Ok(account_id) = parent.parse::<AzureCognitiveServicesAccountResourceId>() {
                            AzureCognitiveServicesAccountDeploymentListRequest {
                                authentication: self.authentication,
                                account_id,
                            }.execute(&child).invalidate().await
                        } else { Ok(()) }
                    }
                    _ => Ok(()),
                };
                let invalidations = [
                    AzureResourceTagsGetRequest { authentication: self.authentication, id: Cow::Borrowed(&self.id) }.execute(&child).invalidate().await,
                    AzureResourceListRequest { authentication: self.authentication }.execute(&child).invalidate().await,
                    inventory,
                    deployment_inventory,
                ];
                let mut failures = invalidations.into_iter().filter_map(Result::err);
                let invalidation = match failures.next() {
                    None => Ok(()),
                    Some(mut error) => {
                        for other in failures {
                            error = error.wrap_err(format!("another tag-read invalidation failed: {other:#}"));
                        }
                        Err(error)
                    }
                };
                match (execution, invalidation) {
                    (Ok(response), Ok(())) => Ok(response),
                    (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                    (Err(error), Err(invalidation)) => Err(error.wrap_err(format!("tag-read invalidation also failed: {invalidation:#}"))),
                }
            }).await
        })
    }
}
cloud_terrastodon_registry::register_thing!(AzureResourceTagsUpdateRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceTagsUpdateRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureResourceTagsUpdateRequest<'static> => AzureResourceTags, effects = [Write]);
