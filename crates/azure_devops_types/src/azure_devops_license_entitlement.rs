use crate::azure_devops_account_id::AzureDevOpsAccountId;
use crate::azure_devops_user_id::AzureDevOpsUserId;
use crate::prelude::AzureDevOpsDescriptor;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AzureDevOpsLicenseEntitlementStatus {
    Active,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum AzureDevOpsLicenseEntitlementLicense {
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
pub enum AzureDevOpsLicenseEntitlementOrigin {
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseEntitlementAssignmentSource {
    Unknown,
    GroupRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsLicenseEntitlement {
    #[serde(rename = "accountId")]
    pub account_id: AzureDevOpsAccountId,
    #[serde(rename = "assignmentDate")]
    pub assignment_date: DateTime<Utc>,
    #[serde(rename = "assignmentSource")]
    pub assignment_source: AzureDevOpsLicenseEntitlementAssignmentSource,
    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<Utc>,
    #[serde(rename = "lastAccessedDate")]
    pub last_accessed_date: DateTime<Utc>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
    #[serde(rename = "license")]
    pub license: AzureDevOpsLicenseEntitlementLicense,
    #[serde(rename = "origin")]
    pub origin: AzureDevOpsLicenseEntitlementOrigin,
    #[serde(rename = "status")]
    pub status: AzureDevOpsLicenseEntitlementStatus,
    #[serde(rename = "user")]
    pub user: AzureDevOpsLicenseEntitlementUserReference,
    #[serde(rename = "userId")]
    pub user_id: AzureDevOpsUserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsLicenseEntitlementUserReference {
    #[serde(rename = "descriptor")]
    pub descriptor: AzureDevOpsDescriptor,
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
