use crate::AzureRoleDefinitionResourceProperties;
use crate::ResourceType;
use crate::RoleDefinitionId;
use crate::RoleDefinitionName;
use arbitrary::Arbitrary;
use facet::Facet;

/// An ARM [role definition](https://learn.microsoft.com/en-us/rest/api/authorization/role-definitions/list?view=rest-authorization-2022-04-01#roledefinition).
#[derive(Debug, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureRoleDefinitionResource {
    pub id: RoleDefinitionId,
    pub name: RoleDefinitionName,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    pub properties: AzureRoleDefinitionResourceProperties,
}

cloud_terrastodon_registry::register_thing!(AzureRoleDefinitionResource);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleDefinitionResource);
