use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsGroup;
use arbitrary::Arbitrary;
use facet::Facet;

/// A permission group contributing to a discovered project membership.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsProjectPermissionObject {
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub principal_name: String,
    pub origin: String,
    pub origin_id: String,
    pub subject_kind: String,
}

impl From<&AzureDevOpsGroup> for AzureDevOpsProjectPermissionObject {
    fn from(group: &AzureDevOpsGroup) -> Self {
        AzureDevOpsProjectPermissionObject {
            descriptor: group.descriptor.clone(),
            display_name: group.display_name.clone(),
            principal_name: group.principal_name.clone(),
            origin: group.origin.clone(),
            origin_id: group.origin_id.clone(),
            subject_kind: group.subject_kind.clone(),
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectPermissionObject);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectPermissionObject);
