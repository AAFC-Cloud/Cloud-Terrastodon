use crate::AzureDevOpsJsonPatchOperation;
use crate::AzureDevOpsWorkItemPatchFields;
use crate::AzureDevOpsWorkItemRelationInput;
use crate::WORK_ITEM_PARENT_RELATION;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;

/// A non-empty Azure DevOps JSON Patch document.
#[derive(Debug, Clone, facet::Facet)]
#[facet(proxy = Vec<AzureDevOpsJsonPatchOperation>)]
pub struct AzureDevOpsJsonPatch(Vec<AzureDevOpsJsonPatchOperation>);

impl<'a> Arbitrary<'a> for AzureDevOpsJsonPatch {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut operations = Vec::<AzureDevOpsJsonPatchOperation>::arbitrary(u)?;
        if operations.is_empty() {
            operations.push(AzureDevOpsJsonPatchOperation::arbitrary(u)?);
        }
        Ok(Self(operations))
    }
}

impl TryFrom<Vec<AzureDevOpsJsonPatchOperation>> for AzureDevOpsJsonPatch {
    type Error = eyre::Report;

    fn try_from(operations: Vec<AzureDevOpsJsonPatchOperation>) -> Result<Self> {
        Self::new(operations)
    }
}

impl AzureDevOpsJsonPatch {
    /// Builds the JSON Patch document for creating a work item from typed
    /// fields and relation inputs.
    pub fn work_item_create_patch(
        fields: AzureDevOpsWorkItemPatchFields,
        relations: &[AzureDevOpsWorkItemRelationInput],
    ) -> Result<Self> {
        let title = fields
            .get("System.Title")
            .ok_or_else(|| eyre::eyre!("Creation requires System.Title"))?;
        ensure!(
            facet_json::from_str::<String>(title.as_ref()).is_ok_and(|s| !s.trim().is_empty()),
            "Title must be a nonempty string"
        );
        ensure!(
            !fields.contains_key("System.Parent"),
            "Set the parent with --parent or a Parent relation"
        );
        ensure!(
            relations
                .iter()
                .filter(|relation| relation.rel.as_ref() == WORK_ITEM_PARENT_RELATION)
                .count()
                <= 1,
            "An item may have only one Parent relation"
        );
        let mut patch: Vec<_> = fields.into_patch_operations().collect();
        let mut seen = HashSet::new();
        for relation in relations {
            ensure!(
                seen.insert((&relation.rel, &relation.url)),
                "Duplicate relation input"
            );
            patch.push(relation.add_patch()?);
        }
        Self::try_from(patch)
    }

    pub fn new(operations: Vec<AzureDevOpsJsonPatchOperation>) -> Result<Self> {
        ensure!(
            !operations.is_empty(),
            "A JSON Patch document must contain at least one operation"
        );
        Ok(Self(operations))
    }
}

impl From<&AzureDevOpsJsonPatch> for Vec<AzureDevOpsJsonPatchOperation> {
    fn from(patch: &AzureDevOpsJsonPatch) -> Self {
        patch.0.clone()
    }
}

impl Deref for AzureDevOpsJsonPatch {
    type Target = [AzureDevOpsJsonPatchOperation];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AzureDevOpsJsonPatch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AzureDevOpsJsonPatch {
    pub fn push(&mut self, operation: AzureDevOpsJsonPatchOperation) {
        self.0.push(operation);
    }

    pub fn insert(&mut self, index: usize, operation: AzureDevOpsJsonPatchOperation) {
        self.0.insert(index, operation);
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsJsonPatch);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsJsonPatch);
