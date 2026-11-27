use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAccessControlList;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsIdentityPermissionEntry;
use cloud_terrastodon_azure_devops_types::AzureDevOpsIdentityPermissionReport;
use cloud_terrastodon_azure_devops_types::AzureDevOpsLegacyIdentityDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsSecurityIdentity;
use cloud_terrastodon_azure_devops_types::AzureDevOpsSecurityNamespace;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// Reads the permissions explicitly assigned to an identity or its parent groups.
///
/// [Read Identities (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/ims/identities/read-identities?view=azure-devops-rest-7.1)
/// resolves the Graph subject and its expanded parent memberships to legacy descriptors.
/// [Security Namespaces Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/security-namespaces/query?view=azure-devops-rest-7.1)
/// supplies the permission vocabulary for [Access Control Lists Query (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/security/access-control-lists/query?view=azure-devops-rest-7.1).
/// Results depend on the caller's visibility. Azure DevOps evaluates effective
/// access separately using token inheritance and other access policies.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsIdentityPermissionsRequest {
    pub organization: AzureDevOpsOrganizationUrl,
    pub subject: AzureDevOpsDescriptor,
    pub authentication: AzureDevOpsAuthSelection,
}

impl HasCacheKey for AzureDevOpsIdentityPermissionsRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        let subject_descriptor = self.subject.to_string();
        let prefix: String = subject_descriptor.chars().take(32).collect();
        let fingerprint = blake3::hash(subject_descriptor.as_bytes());
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az/devops/org",
            &sanitize_windows_path_component(
                &self.organization.organization_name()?.to_ascii_lowercase(),
            ),
            match self.subject {
                AzureDevOpsDescriptor::EntraGroup(_)
                | AzureDevOpsDescriptor::AzureDevOpsGroup(_) => "group",
                _ => "identity",
            },
            &format!("{}-{fingerprint}", sanitize_windows_path_component(&prefix)),
            "permissions",
        ]))
    }
}

impl InvalidatesOwnCacheKey for AzureDevOpsIdentityPermissionsRequest {}

impl ExecutableRequest for AzureDevOpsIdentityPermissionsRequest {
    type Error = eyre::Report;
    type Output = AzureDevOpsIdentityPermissionReport;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        self.organization.organization_name()?;
        executor.validate_authentication_target(
            &self.authentication.into(),
            &self.organization.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let owner = CacheOperationOwner::of::<Self>();
            let descriptor = self.subject.to_string();
            ensure!(
                !descriptor.is_empty() && !descriptor.contains(','),
                "An Azure DevOps identities query requires one subject descriptor"
            );
            let url = self.organization.api_url(
                "vssps.dev.azure.com",
                "_apis/identities",
                &[
                    ("subjectDescriptors", &descriptor),
                    ("queryMembership", "ExpandedUp"),
                    ("api-version", "7.1"),
                ],
            )?;
            let identities = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(key.join("identity")?)
                .authentication(self.authentication.into())
                .receive_cached::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsSecurityIdentity>>>(
                    &executor, owner,
                )
                .await?;
            let Some(identity) = identities
                .value
                .iter()
                .find(|identity| identity.subject_descriptor == self.subject)
            else {
                bail!(
                    "Azure DevOps returned no legacy identity for subject {}",
                    self.subject
                );
            };
            let parent_groups = identity
                .member_of
                .iter()
                .flatten()
                .cloned()
                .filter(|parent| *parent != identity.descriptor)
                .collect::<BTreeSet<_>>();
            let descriptors = parent_groups
                .iter()
                .chain([&identity.descriptor])
                .collect::<Vec<_>>();
            ensure!(
                descriptors
                    .iter()
                    .all(|descriptor| !descriptor.0.is_empty() && !descriptor.0.contains(',')),
                "An Azure DevOps identity could not be represented in the ACL descriptor filter"
            );
            // Bound descriptor lists to keep the lookup URL short for identities with many parents.
            let descriptor_filters = descriptors
                .chunks(12)
                .map(|batch| {
                    let filter = batch
                        .iter()
                        .map(|descriptor| descriptor.0.as_str())
                        .collect::<Vec<_>>()
                        .join(",");
                    (batch, filter)
                })
                .collect::<Vec<_>>();

            let url = self.organization.api_url(
                "dev.azure.com",
                "_apis/securitynamespaces",
                &[("api-version", "7.1")],
            )?;
            let namespaces = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(key.join("namespaces")?)
                .authentication(self.authentication.into())
                .receive_cached::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsSecurityNamespace>>>(
                    &executor, owner,
                )
                .await?;
            let acl_key = key.join("acl")?;
            let results = (0..namespaces.value.len())
                .map_concurrent(
                    &executor,
                    NonZeroUsize::MIN.saturating_add(3),
                    async |executor, index| {
                        let namespace = &namespaces.value[index];
                        let query = async {
                            let namespace_key = acl_key.join(namespace.namespace_id.to_string())?;
                            let mut entries = Vec::new();
                            for (page, (batch, filter)) in descriptor_filters.iter().enumerate() {
                                executor.cancellation().bail_if_cancelled()?;
                                let url = self.organization.api_url(
                                    "dev.azure.com",
                                    &format!("_apis/accesscontrollists/{}", namespace.namespace_id),
                                    &[("descriptors", filter), ("api-version", "7.1")],
                                )?;
                                let acls =
                                    RestRequest::from_method_and_url(Method::GET, url)?
                                        .cache(namespace_key.join(page.to_string())?)
                                        .authentication(self.authentication.into())
                                        .receive_cached::<AzureDevOpsCollectionResponse<
                                            Vec<AzureDevOpsAccessControlList>,
                                        >>(&executor, owner)
                                        .await?;
                                for acl in &acls.value {
                                    entries.extend(permission_entries(namespace, acl, batch));
                                }
                            }
                            eyre::Ok(entries)
                        };
                        // An inaccessible namespace must not discard successful reads in the same organization.
                        eyre::Ok(query.await)
                    },
                )
                .await?;
            executor.cancellation().bail_if_cancelled()?;

            let mut entries = Vec::new();
            let mut checked_namespace_count = 0;
            let mut failed_namespaces = BTreeMap::new();
            for (namespace, result) in namespaces.value.iter().zip(results) {
                match result {
                    Ok(grants) => {
                        checked_namespace_count += 1;
                        entries.extend(grants);
                    }
                    Err(error) => {
                        let cause: &(dyn Error + 'static) = error.as_ref();
                        tracing::error!(
                            organization = %self.organization,
                            namespace = %namespace.name,
                            namespace_id = %namespace.namespace_id,
                            error = cause,
                            "Azure DevOps security namespace permissions could not be loaded"
                        );
                        let name = format!("{} ({})", namespace.name, namespace.namespace_id);
                        failed_namespaces.insert(name, format!("{error:#}"));
                    }
                }
            }
            Ok(AzureDevOpsIdentityPermissionReport {
                subject: identity.descriptor.clone(),
                parent_groups: parent_groups.into_iter().collect(),
                parent_memberships_available: identity.member_of.is_some(),
                entries,
                checked_namespace_count,
                failed_namespaces,
            })
        })
    }
}

fn permission_entries<'a>(
    namespace: &'a AzureDevOpsSecurityNamespace,
    acl: &'a AzureDevOpsAccessControlList,
    assignees: &'a [&'a AzureDevOpsLegacyIdentityDescriptor],
) -> impl Iterator<Item = AzureDevOpsIdentityPermissionEntry> + 'a {
    acl.aces_dictionary
        .values()
        .filter(move |entry| {
            assignees.contains(&&entry.descriptor) && (entry.allow != 0 || entry.deny != 0)
        })
        .map(|entry| AzureDevOpsIdentityPermissionEntry {
            namespace_id: namespace.namespace_id,
            namespace: namespace.display_name.clone(),
            security_token: acl.token.clone(),
            assignee: entry.descriptor.clone(),
            allow: namespace.action_names(entry.allow),
            deny: namespace.action_names(entry.deny),
        })
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsIdentityPermissionsRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsIdentityPermissionsRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsIdentityPermissionsRequest => AzureDevOpsIdentityPermissionReport,
    effects = [Read]
);
