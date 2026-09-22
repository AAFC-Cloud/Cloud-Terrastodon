use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::BTreeMap;

/// Attributes returned by Azure DevOps for a work item relation.
///
/// Azure DevOps may add relation attributes that are not part of the writable
/// `comment`/`isLocked` input shape, so unknown response attributes are kept in
/// `additional`.
#[derive(Debug, Clone, Default, PartialEq, Eq, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationAttributes {
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub comment: Option<String>,
    #[facet(
        rename = "isLocked",
        default,
        skip_serializing_if = Option::is_none
    )]
    pub is_locked: Option<bool>,
    #[facet(flatten, default)]
    pub additional: BTreeMap<String, ArbitraryJson>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemRelationAttributes);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemRelationAttributes);
