use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::PolicyAssignmentId;
use cloud_terrastodon_azure_types::PolicyDefinitionReferenceId;
use cloud_terrastodon_azure_types::PolicyRemediation;
use cloud_terrastodon_azure_types::PolicyRemediationCreateBody;
use cloud_terrastodon_azure_types::PolicyRemediationCreateProperties;
use cloud_terrastodon_azure_types::PolicyRemediationId;
use cloud_terrastodon_azure_types::PolicyRemediationScope;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::HasCacheKey;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::eyre;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;
use url::Url;

const REMEDIATIONS_API_VERSION: &str = "2024-10-01";

/// Submit an Azure Policy remediation at a resource group or management group.
///
/// Uses the [resource group](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource-group?view=rest-policyinsights-2024-10-01)
/// or [management group](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-management-group?view=rest-policyinsights-2024-10-01)
/// create-or-update operation. Supply a fresh name to create a new task. Existing
/// task settings cannot be changed once its deployments begin.
///
/// This preserves the remediation workflow's omitted settings: Azure discovers
/// existing noncompliant resources, with service defaults for resource count,
/// parallel deployments, and failure threshold. See [task defaults](https://learn.microsoft.com/en-us/azure/governance/policy/concepts/remediation-structure).
/// Success acknowledges submission; it does not wait for deployment completion.
/// No remediation read cache currently exists to invalidate. Resource/compliance
/// inventories must be refreshed after the asynchronous work actually completes.
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct PolicyRemediationCreateRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    pub id: PolicyRemediationId,
    pub policy_assignment_id: PolicyAssignmentId,
    /// Required when the assignment targets an initiative; omitted for a single definition.
    pub policy_definition_reference_id: Option<PolicyDefinitionReferenceId>,
}

/// Submit a remediation task using an explicit authentication tenant.
///
/// See [`PolicyRemediationCreateRequest`] for scope and asynchronous completion behavior.
pub fn create_policy_remediation<'a>(
    id: PolicyRemediationId,
    policy_assignment_id: PolicyAssignmentId,
    policy_definition_reference_id: Option<PolicyDefinitionReferenceId>,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> PolicyRemediationCreateRequest<'a> {
    PolicyRemediationCreateRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
        id,
        policy_assignment_id,
        policy_definition_reference_id,
    }
}

impl<'a> Arbitrary<'a> for PolicyRemediationCreateRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: u.arbitrary()?,
            auth_context: Cow::Owned(u.arbitrary()?),
            id: u.arbitrary()?,
            policy_assignment_id: u.arbitrary()?,
            policy_definition_reference_id: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for PolicyRemediationCreateRequest<'_> {
    fn cache_key(&self) -> CacheKey {
        let mut path = PathBuf::from("az/arm/tenants")
            .join(self.auth_context.tenant_id.to_string())
            .join("policy-remediations");
        match &self.id.scope {
            PolicyRemediationScope::ManagementGroup(id) => {
                path = path
                    .join("management-groups")
                    .join(sanitize_windows_path_component(id.name()));
            }
            PolicyRemediationScope::ResourceGroup(id) => {
                path = path
                    .join("subscriptions")
                    .join(id.subscription_id.to_string())
                    .join("resource-groups")
                    .join(sanitize_windows_path_component(
                        id.resource_group_name.as_ref(),
                    ));
            }
        }
        let mut identity = blake3::Hasher::new();
        identity.update(self.policy_assignment_id.expanded_form().as_bytes());
        identity.update(&[0]);
        if let Some(reference_id) = &self.policy_definition_reference_id {
            identity.update(reference_id.as_str().as_bytes());
        }
        CacheKey {
            path: path
                .join("names")
                .join(sanitize_windows_path_component(self.id.name.as_str()))
                .join("create/rest/api-version")
                .join(REMEDIATIONS_API_VERSION)
                .join(identity.finalize().to_hex().as_str()),
            valid_for: Duration::ZERO,
        }
    }
}

impl<'a> IntoFuture for PolicyRemediationCreateRequest<'a> {
    type Output = Result<PolicyRemediation>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let cache_key = self.cache_key();
            let mut url = Url::parse("https://management.azure.com")
                .wrap_err("constructing the remediation endpoint")?;
            {
                let mut path = url
                    .path_segments_mut()
                    .map_err(|_| eyre!("the ARM endpoint cannot contain path segments"))?;
                match &self.id.scope {
                    PolicyRemediationScope::ManagementGroup(id) => {
                        path.extend([
                            "providers",
                            "Microsoft.Management",
                            "managementGroups",
                            id.name(),
                        ]);
                    }
                    PolicyRemediationScope::ResourceGroup(id) => {
                        path.extend([
                            "subscriptions",
                            &id.subscription_id.to_string(),
                            "resourceGroups",
                            id.resource_group_name.as_ref(),
                        ]);
                    }
                }
                path.extend([
                    "providers",
                    "Microsoft.PolicyInsights",
                    "remediations",
                    self.id.name.as_str(),
                ]);
            }
            url.query_pairs_mut()
                .append_pair("api-version", REMEDIATIONS_API_VERSION);
            let body = PolicyRemediationCreateBody {
                properties: PolicyRemediationCreateProperties {
                    policy_assignment_id: self.policy_assignment_id,
                    policy_definition_reference_id: self.policy_definition_reference_id,
                },
            };
            // RestRequest's PUT deferral retains the response, decoded task,
            // and artifacts even when cancellation arrives after submission.
            RestRequest::from_method_and_url(Method::PUT, url, &self.cancellation)?
                .azure_auth_context(&self.auth_context.auth_context)
                .tenant(self.auth_context.tenant_id)
                .cache(cache_key)
                .body(facet_json::to_string(&body).wrap_err("encoding the remediation request")?)
                .receive::<PolicyRemediation>()
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(PolicyRemediationCreateRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PolicyRemediationCreateRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(
    PolicyRemediationCreateRequest<'static> => PolicyRemediation,
    effects = [Write]
);
