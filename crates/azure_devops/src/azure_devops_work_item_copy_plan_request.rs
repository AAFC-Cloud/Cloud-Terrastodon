use crate::azure_devops_work_item_copy_options::AzureDevOpsWorkItemCopyOptions;
use crate::azure_devops_work_item_copy_plan::AzureDevOpsWorkItemCopyPlan;
use crate::azure_devops_work_item_field_definition_list_request::AzureDevOpsWorkItemFieldDefinitionListRequest;
use crate::azure_devops_work_item_relation_type_list_request::AzureDevOpsWorkItemRelationTypeListRequest;
use crate::azure_devops_work_item_type_field_list_request::AzureDevOpsWorkItemTypeFieldListRequest;
use crate::azure_devops_work_items_get_request::AzureDevOpsWorkItemsGetRequest;
use crate::build_work_item_copy_plan;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::WORK_ITEM_CHILD_RELATION;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsWorkItemCopyPlanRequest<'a> {
    /// Caller-owned cancellation scope, excluded from serialized request input.
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub root: AzureDevOpsWorkItemId,
    pub options: AzureDevOpsWorkItemCopyOptions,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemCopyPlanRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            root: AzureDevOpsWorkItemId::arbitrary(u)?,
            options: AzureDevOpsWorkItemCopyOptions::arbitrary(u)?,
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsWorkItemCopyPlanRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = AzureDevOpsWorkItemCopyPlan;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let as_of = self.options.as_of.unwrap_or_else(chrono::Utc::now);
            let allowed: BTreeSet<_> = self.options.allowed_ids.iter().copied().collect();
            let mut frontier = vec![self.root];
            let mut items = BTreeMap::new();
            while !frontier.is_empty() {
                executor.cancellation().bail_if_cancelled()?;
                // Check every ID before constructing a request, including the root.
                ensure!(
                    allowed.is_empty() || frontier.iter().all(|id| allowed.contains(id)),
                    "Copy discovery reached an item outside --allowed-ids; no changes were made"
                );
                let fetched = AzureDevOpsWorkItemsGetRequest {
                    org_url: Cow::Borrowed(self.org_url.as_ref()),
                    project: Some(self.project.clone()),
                    auth_selection: self.auth_selection,
                    ids: frontier,
                    expand: Default::default(),
                    fields: Vec::new(),
                    as_of: Some(as_of),
                }
                .execute(&executor)
                .await?;
                let mut next = BTreeSet::new();
                for item in fetched {
                    executor.cancellation().bail_if_cancelled()?;
                    if self.options.deep {
                        for relation in item.relations.as_deref().unwrap_or_default() {
                            executor.cancellation().bail_if_cancelled()?;
                            if relation.rel.as_ref() == WORK_ITEM_CHILD_RELATION {
                                next.insert(
                                    relation
                                        .url
                                        .work_item_id_for_organization(self.org_url.as_ref())?,
                                );
                            }
                        }
                    }
                    ensure!(
                        items.insert(item.id, item).is_none(),
                        "Duplicate work item during discovery"
                    );
                }
                frontier = next
                    .into_iter()
                    .filter(|id| !items.contains_key(id))
                    .collect();
            }
            let fields = AzureDevOpsWorkItemFieldDefinitionListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(&executor)
            .await?;
            let relations = AzureDevOpsWorkItemRelationTypeListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(&executor)
            .await?;
            let types: BTreeSet<_> = items
                .values()
                .map(|item| {
                    item.fields
                        .work_item_type
                        .clone()
                        .ok_or_else(|| eyre::eyre!("Missing System.WorkItemType field"))
                })
                .collect::<Result<_>>()?;
            let mut type_fields = BTreeMap::new();
            for work_item_type in types {
                executor.cancellation().bail_if_cancelled()?;
                let metadata = AzureDevOpsWorkItemTypeFieldListRequest {
                    org_url: Cow::Borrowed(self.org_url.as_ref()),
                    project: self.project.clone(),
                    auth_selection: self.auth_selection,
                    work_item_type: work_item_type.clone(),
                }
                .execute(&executor)
                .await?;
                type_fields.insert(work_item_type, metadata);
            }
            executor.cancellation().bail_if_cancelled()?;
            let plan = build_work_item_copy_plan(
                self.org_url.as_ref(),
                &self.project,
                self.root,
                &self.options,
                as_of,
                items.into_values().collect(),
                &fields,
                &type_fields,
                &relations,
            )?;
            executor.cancellation().bail_if_cancelled()?;
            Ok(plan)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemCopyPlanRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemCopyPlanRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsWorkItemCopyPlanRequest<'static> => AzureDevOpsWorkItemCopyPlan,
    effects = [Read]
);
