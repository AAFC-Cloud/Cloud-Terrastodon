use crate::prelude::PrincipalId;
use crate::prelude::RoleDefinitionId;
use crate::prelude::to_iso8601;
use crate::role_eligibility_schedules::RoleEligibilityScheduleId;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleAssignmentScheduleRequest {
    #[serde(rename = "Properties")]
    properties: RoleAssignmentScheduleRequestProperties,
}
impl RoleAssignmentScheduleRequest {
    pub fn new_self_activation(
        principal_id: PrincipalId,
        role_definition_id: RoleDefinitionId,
        role_eligibility_schedule_id: RoleEligibilityScheduleId,
        justification: String,
        duration: Duration,
    ) -> Self {
        let duration = to_iso8601(duration);
        Self {
            properties: RoleAssignmentScheduleRequestProperties {
                principal_id,
                role_definition_id,
                request_type: RoleAssignmentScheduleRequestPropertiesRequestType::SelfActivate,
                linked_role_eligibility_schedule_id: role_eligibility_schedule_id,
                justification,
                schedule_info: RoleAssignmentScheduleRequestPropertiesScheduleInfo {
                    start_date_time: None,
                    expiration: RoleAssignmentScheduleRequestPropertiesScheduleInfoExpiration::AfterDuration { duration },
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleAssignmentScheduleRequestProperties {
    #[serde(rename = "PrincipalId")]
    principal_id: PrincipalId,
    #[serde(rename = "RoleDefinitionId")]
    role_definition_id: RoleDefinitionId,
    #[serde(rename = "RequestType")]
    request_type: RoleAssignmentScheduleRequestPropertiesRequestType,
    #[serde(rename = "LinkedRoleEligibilityScheduleId")]
    linked_role_eligibility_schedule_id: RoleEligibilityScheduleId,
    #[serde(rename = "Justification")]
    justification: String,
    #[serde(rename = "ScheduleInfo")]
    schedule_info: RoleAssignmentScheduleRequestPropertiesScheduleInfo,
    #[serde(rename = "TicketInfo")]
    ticket_info: RoleAssignmentScheduleRequestPropertiesTicketInfo,
    #[serde(rename = "IsValidationOnly")]
    is_validation_only: bool,
    #[serde(rename = "IsActivativation")]
    is_activativation: bool,
}

// https://learn.microsoft.com/en-us/azure/templates/microsoft.authorization/roleassignmentschedulerequests?pivots=deployment-language-terraform#roleassignmentschedulerequestproperties-2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleAssignmentScheduleRequestPropertiesScheduleInfo {
    #[serde(rename = "StartDateTime")]
    start_date_time: Option<DateTime<Utc>>,
    #[serde(rename = "Expiration")]
    expiration: RoleAssignmentScheduleRequestPropertiesScheduleInfoExpiration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "Type")]
pub enum RoleAssignmentScheduleRequestPropertiesScheduleInfoExpiration {
    AfterDateTime {
        #[serde(rename = "EndDateTime")]
        end_date_time: DateTime<Utc>,
    },
    AfterDuration {
        #[serde(rename = "Duration")]
        duration: iso8601_duration::Duration,
    },
    NoExpiration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleAssignmentScheduleRequestPropertiesTicketInfo {
    #[serde(rename = "TicketNumber")]
    ticket_number: String,
    #[serde(rename = "TicketSystem")]
    ticket_system: String,
}
