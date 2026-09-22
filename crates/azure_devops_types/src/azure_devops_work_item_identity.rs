use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsUserId;
use crate::AzureDevOpsWorkItemIdentityLinks;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::BTreeMap;

/// The identity object used by Azure DevOps audit fields such as
/// `System.CreatedBy` and `System.ChangedBy`.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemIdentity {
    #[facet(rename = "_links")]
    pub links: Option<AzureDevOpsWorkItemIdentityLinks>,
    pub descriptor: Option<AzureDevOpsDescriptor>,
    pub display_name: Option<String>,
    pub id: Option<AzureDevOpsUserId>,
    pub image_url: Option<String>,
    pub unique_name: Option<String>,
    pub url: Option<String>,
    #[facet(flatten, default)]
    pub additional: BTreeMap<String, ArbitraryJson>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemIdentity);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemIdentity);
