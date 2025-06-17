use crate::all_or::AllOr;
use crate::app::AppId;
use crate::groups::GroupId;
use crate::prelude::ConditionalAccessNamedLocationId;
use crate::prelude::ConditionalAccessPolicyId;
use crate::users::UserId;
use chrono::DateTime;
use chrono::Utc;
use compact_str::CompactString;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessPolicy {
    pub id: ConditionalAccessPolicyId,
    pub template_id: Option<Value>,
    pub display_name: CompactString,
    pub created_date_time: Option<DateTime<Utc>>,
    pub modified_date_time: Option<DateTime<Utc>>,
    pub state: ConditionalAccessPolicyState,
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub partial_enablement_strategy: Option<Value>,
    pub session_controls: Option<Value>,
    pub conditions: ConditionalAccessPolicyConditions,
    pub grant_controls: Option<ConditionalAccessPolicyGrantControls>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionalAccessPolicyState {
    Enabled,
    Disabled,
    EnabledForReportingButNotEnforced,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditions {
    pub user_risk_levels: Vec<Value>,
    pub sign_in_risk_levels: Vec<Value>,
    pub client_app_types: Vec<Value>,
    pub platforms: Option<Value>,
    pub times: Option<Value>,
    pub device_states: Option<Value>,
    pub devices: Option<Value>,
    pub client_applications: Option<Value>,
    pub applications: ConditionalAccessPolicyConditionsApplications,
    pub users: ConditionalAccessPolicyConditionsUsers,
    pub locations: Option<ConditionalAccessPolicyConditionsLocations>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditionsApplications {
    pub include_applications: Vec<AllOr<AppId>>,
    pub exclude_applications: Vec<AllOr<AppId>>,
    pub include_user_actions: Vec<Value>,
    pub include_authentication_context_class_references: Vec<Value>,
    pub application_filter: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditionsUsers {
    pub include_users: Vec<AllOr<UserId>>,
    pub exclude_users: Vec<AllOr<UserId>>,
    pub include_groups: Vec<AllOr<GroupId>>,
    pub exclude_groups: Vec<AllOr<GroupId>>,
    pub include_roles: Vec<AllOr<Uuid>>, // TODO: dedicated type for entra role definition IDs
    pub exclude_roles: Vec<AllOr<Uuid>>, // TODO: dedicated type for entra role definition IDs
    pub include_guests_or_external_users: Option<Value>,
    pub exclude_guests_or_external_users: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditionsLocations {
    pub include_locations: Vec<AllOr<ConditionalAccessNamedLocationId>>,
    pub exclude_locations: Vec<AllOr<ConditionalAccessNamedLocationId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyGrantControls {
    pub operator: ConditionalAccessPolicyGrantControlOperator,
    pub built_in_controls: Vec<ConditionalAccessPolicyGrantControlBuiltInControl>,
    pub custom_authentication_factors: Vec<Value>,
    pub terms_of_use: Vec<Value>,
    #[serde(rename = "authenticationStrength@odata.context")]
    pub authentication_strength_context: String,
    pub authentication_strength: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConditionalAccessPolicyGrantControlOperator {
    And,
    Or,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionalAccessPolicyGrantControlBuiltInControl {
    Block,
    Mfa,
    CompliantDevice,
    DomainJoinedDevice,
    ApprovedApplication,
    CompliantApplication,
    PasswordChange,
    UnknownFutureValue,
}
