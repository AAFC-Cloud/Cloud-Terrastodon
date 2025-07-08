use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use crate::azure_devops_account_id::AzureDevOpsAccountId;
use crate::azure_devops_user_id::AzureDevOpsUserId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AzureDevOpsUserEntitlementStatus {
    Active,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum AzureDevOpsUserEntitlementLicense {
    #[serde(rename = "Account-Express")]
    AccountExpress,
    #[serde(rename = "Account-Stakeholder")]
    AccountStakeholder,
    #[serde(rename = "Account-Advanced")]
    AccountAdvanced,
    #[serde(rename = "Msdn-Enterprise")]
    MsdnEnterprise,
    #[serde(rename = "Msdn-Professional")]
    MsdnProfessional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AzureDevOpsUserEntitlementOrigin {
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsUserEntitlementAssignmentSource {
    Unknown,
    GroupRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsUserEntitlement {
    #[serde(rename = "accountId")]
    pub account_id: AzureDevOpsAccountId,
    #[serde(rename = "assignmentDate")]
    pub assignment_date: DateTime<Utc>,    #[serde(rename = "assignmentSource")]
    pub assignment_source: AzureDevOpsUserEntitlementAssignmentSource,
    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<Utc>,
    #[serde(rename = "lastAccessedDate")]
    pub last_accessed_date: DateTime<Utc>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
    #[serde(rename = "license")]
    pub license: AzureDevOpsUserEntitlementLicense,
    #[serde(rename = "origin")]
    pub origin: AzureDevOpsUserEntitlementOrigin,
    #[serde(rename = "status")]
    pub status: AzureDevOpsUserEntitlementStatus,
    #[serde(rename = "user")]
    pub user: AzureDevOpsUserEntitlementUserReference,
    #[serde(rename = "userId")]
    pub user_id: AzureDevOpsUserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsUserEntitlementUserReference {
    #[serde(rename = "descriptor")]
    pub descriptor: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "id")]
    pub id: AzureDevOpsUserId,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "uniqueName")]
    pub unique_name: String,
    #[serde(rename = "url")]
    pub url: String,
}