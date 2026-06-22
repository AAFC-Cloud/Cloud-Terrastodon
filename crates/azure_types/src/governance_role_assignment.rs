/// See the following:
/// - https://graph.microsoft.com/v1.0/$metadata
/// - https://graph.microsoft.com/beta/$metadata
use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, facet::Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentMemberType {
    Group,
    Direct,
}
#[derive(Debug, facet::Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentStatus {
    Provisioned,
    Accepted,
}

#[derive(Debug, facet::Facet)]
#[repr(C)]
pub enum GovernanceRoleAssignmentState {
    Active,
    Eligible,
}

#[derive(Debug, facet::Facet)]
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
