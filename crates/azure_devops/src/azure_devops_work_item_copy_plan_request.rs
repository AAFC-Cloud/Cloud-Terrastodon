use crate::azure_devops_work_item_copy_options::AzureDevOpsWorkItemCopyOptions;
use crate::azure_devops_work_item_copy_plan::AzureDevOpsWorkItemCopyPlan;
use crate::azure_devops_work_item_field_definition_list_request::AzureDevOpsWorkItemFieldDefinitionListRequest;
use crate::azure_devops_work_item_relation_type_list_request::AzureDevOpsWorkItemRelationTypeListRequest;
use crate::azure_devops_work_item_type_field_list_request::AzureDevOpsWorkItemTypeFieldListRequest;
use crate::azure_devops_work_items_get_request::AzureDevOpsWorkItemsGetRequest;
use crate::build_work_item_copy_plan;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use cloud_terrastodon_azure_devops_types::WORK_ITEM_CHILD_RELATION;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::future::Future;
use std::future::IntoFuture;
use std::pin::Pin;

#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsWorkItemCopyPlanRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub root: AzureDevOpsWorkItemId,
    pub options: AzureDevOpsWorkItemCopyOptions,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemCopyPlanRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            root: AzureDevOpsWorkItemId::arbitrary(u)?,
            options: AzureDevOpsWorkItemCopyOptions::arbitrary(u)?,
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsWorkItemCopyPlanRequest<'a> {
    type Output = Result<AzureDevOpsWorkItemCopyPlan>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let as_of = self.options.as_of.clone().unwrap_or_else(chrono::Utc::now);
            let allowed: BTreeSet<_> = self.options.allowed_ids.iter().copied().collect();
            let mut frontier = vec![self.root];
            let mut items = BTreeMap::new();
            while !frontier.is_empty() {
                // Check every ID before constructing a request, including the root.
                ensure!(
                    allowed.is_empty() || frontier.iter().all(|id| allowed.contains(id)),
                    "Copy discovery reached an item outside --allowed-ids; no changes were made"
                );
                let fetched = AzureDevOpsWorkItemsGetRequest {
                    org_url: Cow::Borrowed(self.org_url.as_ref()),
                    project: Some(self.project.clone()),
                    auth_context: Cow::Borrowed(self.auth_context.as_ref()),
                    ids: frontier,
                    expand: Default::default(),
                    fields: Vec::new(),
                    as_of: Some(as_of.clone()),
                }
                .await?;
                let mut next = BTreeSet::new();
                for item in fetched {
                    if self.options.deep {
                        for relation in item.relations.as_deref().unwrap_or_default() {
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
                auth_context: Cow::Borrowed(self.auth_context.as_ref()),
            }
            .await?;
            let relations = AzureDevOpsWorkItemRelationTypeListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_context: Cow::Borrowed(self.auth_context.as_ref()),
            }
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
                let metadata = AzureDevOpsWorkItemTypeFieldListRequest {
                    org_url: Cow::Borrowed(self.org_url.as_ref()),
                    project: self.project.clone(),
                    auth_context: Cow::Borrowed(self.auth_context.as_ref()),
                    work_item_type: work_item_type.clone(),
                }
                .await?;
                type_fields.insert(work_item_type, metadata);
            }
            build_work_item_copy_plan(
                self.org_url.as_ref(),
                &self.project,
                self.root,
                &self.options,
                as_of,
                items.into_values().collect(),
                &fields,
                &type_fields,
                &relations,
            )
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemCopyPlanRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemCopyPlanRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsWorkItemCopyPlanRequest<'static> => AzureDevOpsWorkItemCopyPlan,
    effects = [Read]
);
