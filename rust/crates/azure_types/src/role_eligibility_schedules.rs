use crate::prelude::RoleDefinitionId;
use crate::prelude::RoleDefinitionKind;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromManagementGroupScoped;
use chrono::DateTime;
use chrono::Utc;
use eyre::Result;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use uuid::Uuid;

pub const ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX: &str =
    "/providers/Microsoft.Authorization/roleEligibilitySchedules/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RoleEligibilityScheduleId {
    ManagementGroupScoped { expanded: String },
}
impl NameValidatable for RoleEligibilityScheduleId {
    fn validate_name(name: &str) -> Result<()> {
        Uuid::parse_str(name)?;
        Ok(())
    }
}
impl HasPrefix for RoleEligibilityScheduleId {
    fn get_prefix() -> &'static str {
        ROLE_ELIGIBILITY_SCHEDULE_ID_PREFIX
    }
}

impl TryFromManagementGroupScoped for RoleEligibilityScheduleId {
    unsafe fn new_management_group_scoped_unchecked(expanded: &str) -> Self {
        RoleEligibilityScheduleId::ManagementGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}
impl Scope for RoleEligibilityScheduleId {
    fn expanded_form(&self) -> &str {
        match self {
            Self::ManagementGroupScoped { expanded } => expanded,
        }
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        RoleEligibilityScheduleId::try_from_expanded_management_group_scoped(expanded)
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RoleEligibilitySchedule
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::RoleEligibilitySchedule(self.clone())
    }
}

impl Serialize for RoleEligibilityScheduleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for RoleEligibilityScheduleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = RoleEligibilityScheduleId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

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

impl HasScope for RoleEligibilitySchedule {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &RoleEligibilitySchedule {
    fn scope(&self) -> &impl Scope {
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
// impl From<RoleEligibilitySchedule> for TofuImportBlock {
//     fn from(role_eligibility_schedule: RoleEligibilitySchedule) -> Self {
//         TofuImportBlock {
//             provider: TofuProviderReference::Inherited,
//             id: role_eligibility_schedule.id.to_string(),
//             to: TofuResourceReference::AzureRM {
//                 kind: TofuAzureRMResourceKind::RoleEligibilitySchedule,
//                 name: format!("{}__{}", role_eligibility_schedule.name, role_eligibility_schedule.id).sanitize(),
//             },
//         }
//     }
// }
