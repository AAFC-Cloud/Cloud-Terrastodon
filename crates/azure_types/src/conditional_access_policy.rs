use crate::ArbitraryJson;
use crate::ConditionalAccessNamedLocationId;
use crate::ConditionalAccessPolicyId;
use crate::EntraGroupId;
use crate::EntraUserId;
use crate::all_or::AllOr;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use compact_str::CompactString;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessPolicy {
    pub id: ConditionalAccessPolicyId,
    pub template_id: Option<ArbitraryJson>,
    pub display_name: CompactString,
    pub created_date_time: Option<DateTime<Utc>>,
    pub modified_date_time: Option<DateTime<Utc>>,
    pub state: ConditionalAccessPolicyState,
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub partial_enablement_strategy: Option<ArbitraryJson>,
    pub session_controls: Option<ArbitraryJson>,
    pub conditions: ConditionalAccessPolicyConditions,
    pub grant_controls: Option<ConditionalAccessPolicyGrantControls>,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum ConditionalAccessPolicyState {
    Enabled,
    Disabled,
    EnabledForReportingButNotEnforced,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditions {
    pub user_risk_levels: Vec<ArbitraryJson>,
    pub sign_in_risk_levels: Vec<ArbitraryJson>,
    pub client_app_types: Vec<ArbitraryJson>,
    pub platforms: Option<ArbitraryJson>,
    pub times: Option<ArbitraryJson>,
    pub device_states: Option<ArbitraryJson>,
    pub devices: Option<ArbitraryJson>,
    pub client_applications: Option<ArbitraryJson>,
    pub applications: ConditionalAccessPolicyConditionsApplications,
    pub users: ConditionalAccessPolicyConditionsUsers,
    pub locations: Option<ConditionalAccessPolicyConditionsLocations>,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditionsApplications {
    #[facet(proxy = crate::AllOrVecProxy)]
    pub include_applications: Vec<AllOr<String>>, // commonly a Uuid, but may be a literal like "Office365"
    #[facet(proxy = crate::AllOrVecProxy)]
    pub exclude_applications: Vec<AllOr<String>>, // commonly a Uuid, but may be a literal like "Office365"
    pub include_user_actions: Vec<ArbitraryJson>,
    pub include_authentication_context_class_references: Vec<ArbitraryJson>,
    pub application_filter: Option<ArbitraryJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditionsUsers {
    #[facet(proxy = crate::AllOrVecProxy)]
    pub include_users: Vec<AllOr<EntraUserId>>,
    #[facet(proxy = crate::AllOrVecProxy)]
    pub exclude_users: Vec<AllOr<EntraUserId>>,
    #[facet(proxy = crate::AllOrVecProxy)]
    pub include_groups: Vec<AllOr<EntraGroupId>>,
    #[facet(proxy = crate::AllOrVecProxy)]
    pub exclude_groups: Vec<AllOr<EntraGroupId>>,
    #[facet(proxy = crate::AllOrVecProxy)]
    pub include_roles: Vec<AllOr<Uuid>>, // TODO: dedicated type for entra role definition IDs
    #[facet(proxy = crate::AllOrVecProxy)]
    pub exclude_roles: Vec<AllOr<Uuid>>, // TODO: dedicated type for entra role definition IDs
    pub include_guests_or_external_users: Option<ArbitraryJson>,
    pub exclude_guests_or_external_users: Option<ArbitraryJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyConditionsLocations {
    #[facet(proxy = crate::AllOrVecProxy)]
    pub include_locations: Vec<AllOr<ConditionalAccessNamedLocationId>>,
    #[facet(proxy = crate::AllOrVecProxy)]
    pub exclude_locations: Vec<AllOr<ConditionalAccessNamedLocationId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessPolicyGrantControls {
    pub operator: ConditionalAccessPolicyGrantControlOperator,
    pub built_in_controls: Vec<ConditionalAccessPolicyGrantControlBuiltInControl>,
    pub custom_authentication_factors: Vec<ArbitraryJson>,
    pub terms_of_use: Vec<ArbitraryJson>,
    #[facet(rename = "authenticationStrength@odata.context")]
    pub authentication_strength_context: String,
    pub authentication_strength: Option<ArbitraryJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "UPPERCASE")]
#[repr(C)]
pub enum ConditionalAccessPolicyGrantControlOperator {
    And,
    Or,
}
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
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

cloud_terrastodon_registry::register_thing!(ConditionalAccessPolicy);
cloud_terrastodon_registry::register_arbitrary!(ConditionalAccessPolicy);
cloud_terrastodon_registry::register_arbitrary!(Vec<ConditionalAccessPolicy>);
