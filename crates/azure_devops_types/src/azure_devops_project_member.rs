use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsProjectPermissionObject;
use arbitrary::Arbitrary;
use facet::Facet;

/// A project member and the permission objects through which membership was discovered.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsProjectMember {
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub principal_name: String,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub subject_kind: String,
    pub permission_objects: Vec<AzureDevOpsProjectPermissionObject>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectMember);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectMember);
