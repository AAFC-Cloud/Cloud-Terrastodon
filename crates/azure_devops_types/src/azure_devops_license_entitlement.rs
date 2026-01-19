use crate::azure_devops_account_id::AzureDevOpsAccountId;
use crate::azure_devops_user_id::AzureDevOpsUserId;
use crate::prelude::AzureDevOpsDescriptor;
use crate::prelude::AzureDevOpsLicenseKind;
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
#[serde(rename_all = "lowercase")]
pub enum AzureDevOpsLicenseEntitlementOrigin {
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum AzureDevOpsLicenseAssignmentSource {
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
    pub assignment_source: AzureDevOpsLicenseAssignmentSource,
    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<Utc>,
    #[serde(rename = "lastAccessedDate")]
    pub last_accessed_date: LastAccessedDate,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
    #[serde(rename = "license")]
    pub license: AzureDevOpsLicenseKind,
    #[serde(rename = "origin")]
    pub origin: AzureDevOpsLicenseEntitlementOrigin,
    #[serde(rename = "status")]
    pub status: AzureDevOpsLicenseEntitlementStatus,
    #[serde(rename = "user")]
    pub user: AzureDevOpsLicenseEntitlementUserReference,
    #[serde(rename = "userId")]
    pub user_id: AzureDevOpsUserId,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LastAccessedDate {
    Some(DateTime<Utc>),
    Never,
}
impl<'de> Deserialize<'de> for LastAccessedDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Step 1: deserialize into a String
        let s = String::deserialize(deserializer)?;

        // Step 2: if value eq "0001-01-01T00:00:00+00:00" then return LastAccessedDate::Never
        if s == "0001-01-01T00:00:00+00:00" {
            return Ok(LastAccessedDate::Never);
        }
        let dt = DateTime::parse_from_rfc3339(&s)
            .map_err(serde::de::Error::custom)?
            .with_timezone(&Utc);
        Ok(LastAccessedDate::Some(dt))
    }
}
impl Serialize for LastAccessedDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LastAccessedDate::Some(dt) => serializer.serialize_str(&dt.to_rfc3339()),
            LastAccessedDate::Never => serializer.serialize_str("0001-01-01T00:00:00+00:00"),
        }
    }
}
#[cfg(test)]
mod test {
    use chrono::DateTime;
    use chrono::Utc;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let x = serde_json::from_str::<super::LastAccessedDate>(r#""0001-01-01T00:00:00+00:00""#)?;
        assert_eq!(x, super::LastAccessedDate::Never);
        Ok(())
    }

    #[test]
    pub fn it_works2() -> eyre::Result<()> {
        let x = serde_json::from_str::<super::LastAccessedDate>(r#""2023-10-05T12:34:56+00:00""#)?;
        assert_eq!(
            x,
            super::LastAccessedDate::Some("2023-10-05T12:34:56+00:00".parse::<DateTime<Utc>>()?)
        );
        Ok(())
    }
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
