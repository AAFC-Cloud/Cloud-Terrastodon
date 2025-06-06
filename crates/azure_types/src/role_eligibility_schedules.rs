use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleDefinitionKind;
use crate::prelude::RoleEligibilityScheduleId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum RoleEligibilityScheduleMemberType {
    Group,
    Direct,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum RoleEligibilitySchedulePrincipalType {
    Group,
    User,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum RoleEligibilityScheduleStatus {
    Provisioned,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleEligibilityScheduleExpandedPropertiesPrincipal {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: Uuid,
    #[serde(rename = "type")]
    pub kind: RoleEligibilitySchedulePrincipalType,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleEligibilityScheduleExpandedPropertiesRoleDefinition {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: RoleDefinitionId,
    #[serde(rename = "type")]
    pub kind: RoleDefinitionKind,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleEligibilityScheduleExpandedPropertiesScope {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleEligibilityScheduleExpandedProperties {
    pub principal: RoleEligibilityScheduleExpandedPropertiesPrincipal,
    #[serde(rename = "roleDefinition")]
    pub role_definition: RoleEligibilityScheduleExpandedPropertiesRoleDefinition,
    pub scope: RoleEligibilityScheduleExpandedPropertiesScope,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleEligibilityScheduleProperties {
    #[serde(rename = "createdOn")]
    pub created_on: DateTime<Utc>,
    #[serde(rename = "expandedProperties")]
    pub expanded_properties: RoleEligibilityScheduleExpandedProperties,
    #[serde(rename = "memberType")]
    pub member_type: RoleEligibilityScheduleMemberType,
    #[serde(rename = "principalId")]
    pub principal_id: Uuid,
    #[serde(rename = "principalType")]
    pub principal_type: RoleEligibilitySchedulePrincipalType,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: RoleDefinitionId,
    #[serde(rename = "roleEligibilityScheduleRequestId")]
    pub role_eligibility_schedule_request_id: String,
    #[serde(rename = "scope")]
    pub scope: ScopeImpl,
    #[serde(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    #[serde(rename = "status")]
    pub status: RoleEligibilityScheduleStatus,
    #[serde(rename = "updatedOn")]
    pub updated_on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
