use crate::RoleDefinitionId;
use crate::RoleDefinitionKind;
use crate::RoleEligibilityScheduleId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, PartialEq, facet::Facet)]
#[repr(C)]
pub enum RoleEligibilityScheduleMemberType {
    Group,
    Direct,
}
#[derive(Debug, PartialEq, facet::Facet)]
#[repr(C)]
pub enum RoleEligibilitySchedulePrincipalType {
    Group,
    User,
}
#[derive(Debug, PartialEq, facet::Facet)]
#[repr(C)]
pub enum RoleEligibilityScheduleStatus {
    Provisioned,
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleEligibilityScheduleExpandedPropertiesPrincipal {
    #[facet(rename = "displayName")]
    pub display_name: String,
    pub id: Uuid,
    #[facet(rename = "type")]
    pub kind: RoleEligibilitySchedulePrincipalType,
}
#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleEligibilityScheduleExpandedPropertiesRoleDefinition {
    #[facet(rename = "displayName")]
    pub display_name: String,
    pub id: RoleDefinitionId,
    #[facet(rename = "type")]
    pub kind: RoleDefinitionKind,
}
#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleEligibilityScheduleExpandedPropertiesScope {
    #[facet(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    #[facet(rename = "type")]
    pub kind: String,
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleEligibilityScheduleExpandedProperties {
    pub principal: RoleEligibilityScheduleExpandedPropertiesPrincipal,
    #[facet(rename = "roleDefinition")]
    pub role_definition: RoleEligibilityScheduleExpandedPropertiesRoleDefinition,
    pub scope: RoleEligibilityScheduleExpandedPropertiesScope,
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleEligibilityScheduleProperties {
    #[facet(rename = "createdOn")]
    pub created_on: DateTime<Utc>,
    #[facet(rename = "expandedProperties")]
    pub expanded_properties: RoleEligibilityScheduleExpandedProperties,
    #[facet(rename = "memberType")]
    pub member_type: RoleEligibilityScheduleMemberType,
    #[facet(rename = "principalId")]
    pub principal_id: Uuid,
    #[facet(rename = "principalType")]
    pub principal_type: RoleEligibilitySchedulePrincipalType,
    #[facet(rename = "roleDefinitionId")]
    pub role_definition_id: RoleDefinitionId,
    #[facet(rename = "roleEligibilityScheduleRequestId")]
    pub role_eligibility_schedule_request_id: String,
    #[facet(rename = "scope")]
    pub scope: ScopeImpl,
    #[facet(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    #[facet(rename = "status")]
    pub status: RoleEligibilityScheduleStatus,
    #[facet(rename = "updatedOn")]
    pub updated_on: DateTime<Utc>,
}

#[derive(Debug, PartialEq, facet::Facet)]
pub struct RoleEligibilitySchedule {
    pub id: RoleEligibilityScheduleId,
    pub name: Uuid,
    pub properties: RoleEligibilityScheduleProperties,
}
impl RoleEligibilitySchedule {
    pub fn get_type() -> &'static str {
        "Microsoft.Authorization/roleEligibilitySchedules"
    }
}

impl AsScope for RoleEligibilitySchedule {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &RoleEligibilitySchedule {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for RoleEligibilitySchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "PIM(role={}, principal={}, scope={})",
            &self
                .properties
                .expanded_properties
                .role_definition
                .display_name,
            &self.properties.expanded_properties.principal.display_name,
            &self.properties.expanded_properties.scope.display_name,
        ))
    }
}
