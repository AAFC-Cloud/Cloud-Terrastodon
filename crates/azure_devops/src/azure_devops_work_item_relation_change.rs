use crate::azure_devops_work_item_relation_selector::AzureDevOpsWorkItemRelationSelector;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatch;
use cloud_terrastodon_azure_devops_types::AzureDevOpsJsonPatchOperation;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItem;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInput;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemRelationInputAttributes;
use cloud_terrastodon_azure_devops_types::WORK_ITEM_CHILD_RELATION;
use cloud_terrastodon_azure_devops_types::WORK_ITEM_PARENT_RELATION;
use cloud_terrastodon_azure_types::ArbitraryJson;
use eyre::Result;
use eyre::ensure;

#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemRelationChange {
    Create(AzureDevOpsWorkItemRelationInput),
    Update {
        selector: AzureDevOpsWorkItemRelationSelector,
        attributes: AzureDevOpsWorkItemRelationInputAttributes,
    },
    Remove(AzureDevOpsWorkItemRelationSelector),
}

impl AzureDevOpsWorkItemRelationChange {
    pub fn patch(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        item: &AzureDevOpsWorkItem,
    ) -> Result<AzureDevOpsJsonPatch> {
        match self {
            Self::Create(input) => {
                if input.rel.as_ref() == WORK_ITEM_PARENT_RELATION
                    || input.rel.as_ref() == WORK_ITEM_CHILD_RELATION
                {
                    ensure!(
                        input.url.work_item_id_for_organization(organization)? != item.id,
                        "An item cannot be its own parent or child"
                    );
                }
                ensure!(
                    !item
                        .relations
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .any(|relation| relation.rel == input.rel && relation.url == input.url),
                    "Relation already exists"
                );
                if input.rel.as_ref() == WORK_ITEM_PARENT_RELATION {
                    ensure!(
                        !item
                            .relations
                            .as_deref()
                            .unwrap_or_default()
                            .iter()
                            .any(|r| r.rel.as_ref() == WORK_ITEM_PARENT_RELATION),
                        "Item already has a parent"
                    );
                }
                AzureDevOpsJsonPatch::try_from(vec![input.add_patch()?])
            }
            Self::Update {
                selector,
                attributes,
            } => {
                ensure!(
                    attributes.comment.is_some() || attributes.is_locked.is_some(),
                    "Specify at least one relation attribute"
                );
                let index = selector.index(organization, item)?;
                let mut patch = Vec::new();
                if let Some(comment) = &attributes.comment {
                    patch.push(AzureDevOpsJsonPatchOperation::Add {
                        path: format!("/relations/{index}/attributes/comment"),
                        value: ArbitraryJson::try_from_facet(comment)?,
                    });
                }
                if let Some(is_locked) = attributes.is_locked {
                    patch.push(AzureDevOpsJsonPatchOperation::Add {
                        path: format!("/relations/{index}/attributes/isLocked"),
                        value: ArbitraryJson::try_from_facet(&is_locked)?,
                    });
                }
                AzureDevOpsJsonPatch::try_from(patch)
            }
            Self::Remove(selector) => {
                AzureDevOpsJsonPatch::try_from(vec![AzureDevOpsJsonPatchOperation::Remove {
                    path: format!("/relations/{}", selector.index(organization, item)?),
                }])
            }
        }
    }
}
