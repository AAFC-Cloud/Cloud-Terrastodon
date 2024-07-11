use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum PimEntraRoleAssignmentMemberType {
    Group,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum PimEntraRoleAssignmentStatus {
    Provisioned,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "assignmentState")]
pub enum PimEntraRoleAssignment {
    Eligible(EligiblePimEntraRoleAssignment),
    Active(ActivePimEntraRoleAssignment),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EligiblePimEntraRoleAssignment {
    pub id: String,
    #[serde(rename = "memberType")]
    pub member_type: PimEntraRoleAssignmentMemberType,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: Uuid,
    #[serde(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    pub status: PimEntraRoleAssignmentStatus,
    #[serde(rename = "subjectId")]
    pub subject_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivePimEntraRoleAssignment {
    #[serde(rename = "endDateTime")]
    pub end_date_time: DateTime<Utc>,
    pub id: String,
    #[serde(rename = "linkedEligibleRoleAssignmentId")]
    pub linked_eligible_role_assignment_id: String,
    #[serde(rename = "memberType")]
    pub member_type: PimEntraRoleAssignmentMemberType,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: Uuid,
    #[serde(rename = "startDateTime")]
    pub start_date_time: DateTime<Utc>,
    pub status: PimEntraRoleAssignmentStatus,
    #[serde(rename = "subjectId")]
    pub subject_id: Uuid,
}

impl PimEntraRoleAssignment {
    pub fn id(&self) -> &String {
        match self {
            PimEntraRoleAssignment::Active(x) => &x.id,
            PimEntraRoleAssignment::Eligible(x) => &x.id,
        }
    }
    pub fn member_type(&self) -> &PimEntraRoleAssignmentMemberType {
        match self {
            PimEntraRoleAssignment::Active(x) => &x.member_type,
            PimEntraRoleAssignment::Eligible(x) => &x.member_type,
        }
    }
    pub fn role_definition_id(&self) -> &Uuid {
        match self {
            PimEntraRoleAssignment::Active(x) => &x.role_definition_id,
            PimEntraRoleAssignment::Eligible(x) => &x.role_definition_id,
        }
    }
    pub fn start_date_time(&self) -> &DateTime<Utc> {
        match self {
            PimEntraRoleAssignment::Active(x) => &x.start_date_time,
            PimEntraRoleAssignment::Eligible(x) => &x.start_date_time,
        }
    }
    pub fn status(&self) -> &PimEntraRoleAssignmentStatus {
        match self {
            PimEntraRoleAssignment::Active(x) => &x.status,
            PimEntraRoleAssignment::Eligible(x) => &x.status,
        }
    }
    pub fn subject_id(&self) -> &Uuid {
        match self {
            PimEntraRoleAssignment::Active(x) => &x.subject_id,
            PimEntraRoleAssignment::Eligible(x) => &x.subject_id,
        }
    }
}
