use crate::AzureDenyAssignmentResourceProperties;
use crate::ResourceType;
use crate::ScopeImpl;
use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

/// An ARM [deny assignment](https://learn.microsoft.com/en-us/rest/api/authorization/deny-assignments/list-for-scope?view=rest-authorization-2022-04-01#denyassignment).
#[derive(Debug, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDenyAssignmentResource {
    pub id: ScopeImpl,
    pub name: Uuid,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    pub properties: AzureDenyAssignmentResourceProperties,
}

cloud_terrastodon_registry::register_thing!(AzureDenyAssignmentResource);
cloud_terrastodon_registry::register_arbitrary!(AzureDenyAssignmentResource);
