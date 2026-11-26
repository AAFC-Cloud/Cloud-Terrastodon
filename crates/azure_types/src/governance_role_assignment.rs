use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;
/// See the following:
/// - https://graph.microsoft.com/v1.0/$metadata
/// - https://graph.microsoft.com/beta/$metadata
use uuid::Uuid;

#[derive(Debug, Arbitrary, Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentMemberType {
    Group,
    Direct,
}
#[derive(Debug, Arbitrary, Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentStatus {
    Provisioned,
    Accepted,
}

#[derive(Debug, Arbitrary, Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentState {
    Active,
    Eligible,
}

#[derive(Debug, Arbitrary, Facet)]
pub struct GovernanceRoleAssignment {
    pub id: String,
    #[facet(rename = "linkedEligibleRoleAssignmentId", default)]
    pub linked_eligible_role_assignment_id: Option<String>,
    #[facet(rename = "memberType")]
    pub member_type: GovernanceRoleAssignmentMemberType,
    #[facet(rename = "roleDefinitionId")]
    pub role_definition_id: Uuid,
    #[facet(rename = "startDateTime", default)]
    pub start_date_time: Option<Timestamp>,
    #[facet(rename = "endDateTime", default)]
    pub end_date_time: Option<Timestamp>,
    pub status: GovernanceRoleAssignmentStatus,
    #[facet(rename = "subjectId")]
    pub subject_id: Uuid,
    #[facet(rename = "assignmentState")]
    pub assignment_state: GovernanceRoleAssignmentState,
}

cloud_terrastodon_registry::register_thing!(GovernanceRoleAssignment);
cloud_terrastodon_registry::register_arbitrary!(GovernanceRoleAssignment);
