use crate::AzureDevOpsJsonPatchOperation;
use crate::AzureDevOpsOrganizationUrl;
use crate::AzureDevOpsWorkItemId;
use crate::AzureDevOpsWorkItemRelationInputAttributes;
use crate::AzureDevOpsWorkItemRelationTypeName;
use crate::AzureDevOpsWorkItemRelationUrl;
use crate::AzureDevOpsWorkItemUrl;
use crate::WORK_ITEM_PARENT_RELATION;
use cloud_terrastodon_azure_types::ArbitraryJson;
use eyre::Result;
use std::borrow::Cow;

/// Writable relation input shared by create, relation commands, and copy.
#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationInput {
    pub rel: AzureDevOpsWorkItemRelationTypeName,
    pub url: AzureDevOpsWorkItemRelationUrl,
    #[facet(default)]
    pub attributes: AzureDevOpsWorkItemRelationInputAttributes,
}

impl AzureDevOpsWorkItemRelationInput {
    pub fn parent(
        organization: &AzureDevOpsOrganizationUrl,
        id: AzureDevOpsWorkItemId,
    ) -> Result<Self> {
        Ok(Self {
            rel: WORK_ITEM_PARENT_RELATION.parse()?,
            url: AzureDevOpsWorkItemRelationUrl::from(AzureDevOpsWorkItemUrl::new(
                Cow::Owned(organization.clone()),
                id,
            )),
            attributes: Default::default(),
        })
    }

    pub fn add_patch(&self) -> Result<AzureDevOpsJsonPatchOperation> {
        Ok(AzureDevOpsJsonPatchOperation::Add {
            path: "/relations/-".to_owned(),
            value: ArbitraryJson::try_from_facet(self)?,
        })
    }
}
