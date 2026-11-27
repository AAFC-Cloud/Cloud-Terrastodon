use crate::AzureDevOpsWorkItemQueryInvokeRequest;
use crate::AzureDevOpsWorkItemQuerySelector;
use crate::AzureDevOpsWorkItemsGetRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemExpand;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::borrow::Cow;
use std::num::NonZeroU16;
use std::path::PathBuf;
use std::sync::Arc;

/// Reads the most recently changed work items within an explicit project.
///
/// The [WIQL query](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1)
/// bounds and orders the IDs. [Work Items Batch](https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/work-items/get-work-items-batch?view=azure-devops-rest-7.1)
/// loads their list fields as of that query's snapshot time.
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsProjectWorkItemListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project_id: AzureDevOpsProjectId,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub limit: NonZeroU16,
}

impl AzureDevOpsProjectWorkItemListRequest<'_> {
    pub const DEFAULT_LIMIT: NonZeroU16 = NonZeroU16::MIN.saturating_add(199);
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectWorkItemListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(u.arbitrary()?),
            project_id: u.arbitrary()?,
            auth_selection: u.arbitrary()?,
            limit: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for AzureDevOpsProjectWorkItemListRequest<'_> {
    fn cache_key(&self) -> Result<CacheKey> {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(segment));
            }
        }
        path.push("org");
        path.push(sanitize_windows_path_component(
            &self.org_url.name().to_ascii_lowercase(),
        ));
        path.push("project");
        path.push(self.project_id.to_string());
        CacheKey::from_relative_path(path.join("work-item/list-recent"))
    }
}

impl InvalidatesOwnCacheKey for AzureDevOpsProjectWorkItemListRequest<'_> {}

impl ExecutableRequest for AzureDevOpsProjectWorkItemListRequest<'_> {
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<AzureDevOpsWorkItem>>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        let selection = self.auth_selection.into();
        executor.validate_authentication_target(
            &selection,
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&selection)
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = AuthenticationSelection::from(self.auth_selection);
            let key = self.cache_key()?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key.clone(),
                    CacheOperationOwner::of::<AzureDevOpsProjectWorkItemListRequest<'static>>(),
                    &selection,
                    NonEmptyString::new(format!("latest-{}", self.limit))?,
                    WorkReuse::ReusableRead,
                    move |executor| async move {
                        let observation = ArtifactObservation::now();
                        let query = AzureDevOpsWorkItemQueryInvokeRequest {
                            org_url: Cow::Borrowed(self.org_url.as_ref()),
                            project: Some(self.project_id.into()),
                            auth_selection: self.auth_selection,
                            query: AzureDevOpsWorkItemQuerySelector::Wiql(
                                "SELECT [System.Id] FROM WorkItems WHERE [System.TeamProject] = @Project ORDER BY [System.ChangedDate] DESC, [System.Id] DESC"
                                    .to_owned(),
                            ),
                            top: Some(self.limit.into()),
                        }
                        // Both reads participate in the aggregate's cache lineage.
                        .receive_cached(&executor, key.join("query")?)
                        .await?;
                        ensure!(
                            query.work_items.len() <= usize::from(self.limit.get()),
                            "Azure DevOps returned more work items than requested"
                        );
                        if query.work_items.is_empty() {
                            return Ok(WorkOutput::new(Vec::new(), observation));
                        }
                        let items = AzureDevOpsWorkItemsGetRequest {
                            org_url: self.org_url,
                            project: Some(self.project_id.into()),
                            auth_selection: self.auth_selection,
                            ids: query.work_items.iter().map(|item| item.id).collect(),
                            expand: AzureDevOpsWorkItemExpand::None,
                            fields: [
                                "System.Title",
                                "System.WorkItemType",
                                "System.State",
                                "System.ChangedDate",
                            ]
                            .map(str::to_owned)
                            .into(),
                            as_of: Some(query.as_of),
                        }
                        .load_cached(&executor, key.join("batches")?)
                        .await?;
                        Ok(WorkOutput::new(items, observation))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectWorkItemListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectWorkItemListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsProjectWorkItemListRequest<'static> => Arc<Vec<Arc<AzureDevOpsWorkItem>>>,
    effects = [Read]
);
