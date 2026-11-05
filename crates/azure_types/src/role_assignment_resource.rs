use crate::AzureResourceSystemData;
use crate::ResourceType;
use crate::RoleAssignmentId;
use crate::RoleAssignmentName;
use crate::RoleAssignmentResourceProperties;
use arbitrary::Arbitrary;

/// ARM's [role-assignment resource](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01#roleassignment).
/// This preserves the REST envelope separately from the flattened Resource Graph projection.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct RoleAssignmentResource {
    pub id: RoleAssignmentId,
    pub name: RoleAssignmentName,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    pub properties: RoleAssignmentResourceProperties,
    /// Omitted by the documented create examples.
    pub system_data: Option<AzureResourceSystemData>,
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentResource);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentResource);
