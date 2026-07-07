/// See the following:
/// - https://graph.microsoft.com/v1.0/$metadata
/// - https://graph.microsoft.com/beta/$metadata
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentMemberType {
    Group,
    Direct,
}
#[derive(Debug, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentStatus {
    Provisioned,
    Accepted,
}

#[derive(Debug, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentState {
    Active,
    Eligible,
}

#[derive(Debug, Arbitrary, facet::Facet)]
pub struct GovernanceRoleAssignment {
    pub id: String,
    #[facet(rename = "linkedEligibleRoleAssignmentId", default)]
    pub linked_eligible_role_assignment_id: Option<String>,
    #[facet(rename = "memberType")]
    pub member_type: GovernanceRoleAssignmentMemberType,
    #[facet(rename = "roleDefinitionId")]
    pub role_definition_id: Uuid,
    #[facet(rename = "startDateTime", default)]
    pub start_date_time: Option<DateTime<Utc>>,
    #[facet(rename = "endDateTime", default)]
    pub end_date_time: Option<DateTime<Utc>>,
    pub status: GovernanceRoleAssignmentStatus,
    #[facet(rename = "subjectId")]
    pub subject_id: Uuid,
    #[facet(rename = "assignmentState")]
    pub assignment_state: GovernanceRoleAssignmentState,
}

cloud_terrastodon_registry::register_thing!(GovernanceRoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(GovernanceRoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(Vec<GovernanceRoleAssignment>);
