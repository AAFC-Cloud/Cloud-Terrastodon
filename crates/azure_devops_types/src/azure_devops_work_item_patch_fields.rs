use crate::AzureDevOpsJsonPatchOperation;
use crate::AzureDevOpsWorkItemFieldName;
use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::ops::DerefMut;

/// Arbitrary field values supplied when creating or updating a work item.
#[derive(Debug, Clone, PartialEq, Eq, Default, arbitrary::Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsWorkItemPatchFields(BTreeMap<AzureDevOpsWorkItemFieldName, ArbitraryJson>);

impl AzureDevOpsWorkItemPatchFields {
    /// Converts each field to the JSON Patch operation used by the work item
    /// create and update endpoints.
    pub fn into_patch_operations(self) -> impl Iterator<Item = AzureDevOpsJsonPatchOperation> {
        self.into_iter()
            .map(|(field, value)| AzureDevOpsJsonPatchOperation::Add {
                path: field.json_patch_path(),
                value,
            })
    }
}

impl Deref for AzureDevOpsWorkItemPatchFields {
    type Target = BTreeMap<AzureDevOpsWorkItemFieldName, ArbitraryJson>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AzureDevOpsWorkItemPatchFields {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BTreeMap<AzureDevOpsWorkItemFieldName, ArbitraryJson>>
    for AzureDevOpsWorkItemPatchFields
{
    fn from(fields: BTreeMap<AzureDevOpsWorkItemFieldName, ArbitraryJson>) -> Self {
        Self(fields)
    }
}

impl From<AzureDevOpsWorkItemPatchFields>
    for BTreeMap<AzureDevOpsWorkItemFieldName, ArbitraryJson>
{
    fn from(fields: AzureDevOpsWorkItemPatchFields) -> Self {
        fields.0
    }
}

impl FromIterator<(AzureDevOpsWorkItemFieldName, ArbitraryJson)>
    for AzureDevOpsWorkItemPatchFields
{
    fn from_iter<T: IntoIterator<Item = (AzureDevOpsWorkItemFieldName, ArbitraryJson)>>(
        iter: T,
    ) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for AzureDevOpsWorkItemPatchFields {
    type Item = (AzureDevOpsWorkItemFieldName, ArbitraryJson);
    type IntoIter =
        std::collections::btree_map::IntoIter<AzureDevOpsWorkItemFieldName, ArbitraryJson>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a AzureDevOpsWorkItemPatchFields {
    type Item = (&'a AzureDevOpsWorkItemFieldName, &'a ArbitraryJson);
    type IntoIter =
        std::collections::btree_map::Iter<'a, AzureDevOpsWorkItemFieldName, ArbitraryJson>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut AzureDevOpsWorkItemPatchFields {
    type Item = (&'a AzureDevOpsWorkItemFieldName, &'a mut ArbitraryJson);
    type IntoIter =
        std::collections::btree_map::IterMut<'a, AzureDevOpsWorkItemFieldName, ArbitraryJson>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemPatchFields);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemPatchFields);
