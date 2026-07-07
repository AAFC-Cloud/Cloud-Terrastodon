use crate::PrincipalId;
use crate::RoleDefinitionId;
use crate::RoleEligibilityScheduleId;
use crate::iso8601_duration::IsoDuration;
use chrono::DateTime;
use chrono::Utc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, facet::Facet)]
pub struct RoleAssignmentScheduleRequest {
    #[facet(rename = "Properties")]
    pub properties: RoleAssignmentScheduleRequestProperties,
}
impl RoleAssignmentScheduleRequest {
    pub fn new_self_activation(
        principal_id: PrincipalId,
        role_definition_id: RoleDefinitionId,
        role_eligibility_schedule_id: RoleEligibilityScheduleId,
        justification: String,
        duration: Duration,
    ) -> Self {
        Self {
            properties: RoleAssignmentScheduleRequestProperties {
                principal_id,
                role_definition_id,
                request_type: RoleAssignmentScheduleRequestPropertiesRequestType::SelfActivate,
                linked_role_eligibility_schedule_id: role_eligibility_schedule_id,
                justification,
                schedule_info: RoleAssignmentScheduleRequestPropertiesScheduleInfo {
                    start_date_time: None,
                    expiration: RoleAssignmentScheduleRequestPropertiesScheduleInfoExpiration::AfterDuration { duration: duration.into() },
                },
                ticket_info: RoleAssignmentScheduleRequestPropertiesTicketInfo {
                    ticket_number: "".to_string(),
                    ticket_system: "".to_string(),
                },
                is_validation_only: false,
                is_activativation: true,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, facet::Facet)]
pub struct RoleAssignmentScheduleRequestProperties {
    #[facet(rename = "PrincipalId")]
    pub principal_id: PrincipalId,
    #[facet(rename = "RoleDefinitionId")]
    pub role_definition_id: RoleDefinitionId,
    #[facet(rename = "RequestType")]
    pub request_type: RoleAssignmentScheduleRequestPropertiesRequestType,
    #[facet(rename = "LinkedRoleEligibilityScheduleId")]
    pub linked_role_eligibility_schedule_id: RoleEligibilityScheduleId,
    #[facet(rename = "Justification")]
    pub justification: String,
    #[facet(rename = "ScheduleInfo")]
    pub schedule_info: RoleAssignmentScheduleRequestPropertiesScheduleInfo,
    #[facet(rename = "TicketInfo")]
    pub ticket_info: RoleAssignmentScheduleRequestPropertiesTicketInfo,
    #[facet(rename = "IsValidationOnly")]
    pub is_validation_only: bool,
    #[facet(rename = "IsActivativation")]
    pub is_activativation: bool,
}

// https://learn.microsoft.com/en-us/azure/templates/microsoft.authorization/roleassignmentschedulerequests?pivots=deployment-language-terraform#roleassignmentschedulerequestproperties-2
#[derive(Debug, Clone, PartialEq, facet::Facet)]
#[repr(C)]
pub enum RoleAssignmentScheduleRequestPropertiesRequestType {
    AdminAssign,
    AdminExtend,
    AdminRemove,
    AdminRenew,
    AdminUpdate,
    SelfActivate,
    SelfDeactivate,
    SelfExtend,
    SelfRenew,
}

#[derive(Debug, Clone, PartialEq, facet::Facet)]
pub struct RoleAssignmentScheduleRequestPropertiesScheduleInfo {
    #[facet(rename = "StartDateTime", default)]
    pub start_date_time: Option<DateTime<Utc>>,
    #[facet(rename = "Expiration")]
    pub expiration: RoleAssignmentScheduleRequestPropertiesScheduleInfoExpiration,
}

#[derive(Debug, Clone, PartialEq, facet::Facet)]
#[facet(tag = "Type")]
#[repr(C)]
pub enum RoleAssignmentScheduleRequestPropertiesScheduleInfoExpiration {
    AfterDateTime {
        #[facet(rename = "EndDateTime")]
        end_date_time: DateTime<Utc>,
    },
    AfterDuration {
        #[facet(rename = "Duration")]
        duration: IsoDuration,
    },
    NoExpiration,
}

#[derive(Debug, Clone, PartialEq, facet::Facet)]
pub struct RoleAssignmentScheduleRequestPropertiesTicketInfo {
    #[facet(rename = "TicketNumber")]
    pub ticket_number: String,
    #[facet(rename = "TicketSystem")]
    pub ticket_system: String,
}
