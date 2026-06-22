use crate::AzureDevOpsDescriptor;
use crate::AzureDevOpsLicenseType;
use crate::azure_devops_account_id::AzureDevOpsAccountId;
use crate::azure_devops_user_id::AzureDevOpsUserId;
use chrono::DateTime;
use chrono::Utc;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "lowercase")]
#[repr(C)]
pub enum AzureDevOpsLicenseEntitlementStatus {
    Active,
    Pending,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "lowercase")]
#[repr(C)]
pub enum AzureDevOpsLicenseEntitlementOrigin {
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum AzureDevOpsLicenseAssignmentSource {
    Unknown,
    GroupRule,
}

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsUserLicenseEntitlement {
    #[facet(rename = "accountId")]
    pub account_id: AzureDevOpsAccountId,
    #[facet(rename = "assignmentDate")]
    pub assignment_date: DateTime<Utc>,
    #[facet(rename = "assignmentSource")]
    pub assignment_source: AzureDevOpsLicenseAssignmentSource,
    #[facet(rename = "dateCreated")]
    pub date_created: DateTime<Utc>,
    #[facet(rename = "lastAccessedDate")]
    pub last_accessed_date: LastAccessedDate,
    #[facet(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
    #[facet(rename = "license")]
    pub license: AzureDevOpsLicenseType,
    #[facet(rename = "origin")]
    pub origin: AzureDevOpsLicenseEntitlementOrigin,
    #[facet(rename = "status")]
    pub status: AzureDevOpsLicenseEntitlementStatus,
    #[facet(rename = "user")]
    pub user: AzureDevOpsLicenseEntitlementUserReference,
    #[facet(rename = "userId")]
    pub user_id: AzureDevOpsUserId,
}

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum LastAccessedDate {
    Some(DateTime<Utc>),
    Never,
}

impl FromStr for LastAccessedDate {
    type Err = chrono::ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "0001-01-01T00:00:00+00:00" {
            return Ok(LastAccessedDate::Never);
        }

        Ok(LastAccessedDate::Some(
            DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc),
        ))
    }
}

impl From<&LastAccessedDate> for String {
    fn from(value: &LastAccessedDate) -> Self {
        match value {
            LastAccessedDate::Some(dt) => dt.to_rfc3339(),
            LastAccessedDate::Never => "0001-01-01T00:00:00+00:00".to_owned(),
        }
    }
}

impl TryFrom<String> for LastAccessedDate {
    type Error = chrono::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod test {
    use crate::azure_devops_user_license_entitlement;
    use chrono::DateTime;
    use chrono::Utc;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let x = facet_json::from_str::<super::LastAccessedDate>(r#""0001-01-01T00:00:00+00:00""#)?;
        assert_eq!(
            x,
            azure_devops_user_license_entitlement::LastAccessedDate::Never
        );
        Ok(())
    }

    #[test]
    pub fn it_works2() -> eyre::Result<()> {
        let x = facet_json::from_str::<super::LastAccessedDate>(r#""2023-10-05T12:34:56+00:00""#)?;
        assert_eq!(
            x,
            azure_devops_user_license_entitlement::LastAccessedDate::Some(
                "2023-10-05T12:34:56+00:00".parse::<DateTime<Utc>>()?
            )
        );
        Ok(())
    }
}

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsLicenseEntitlementUserReference {
    #[facet(rename = "descriptor")]
    pub descriptor: AzureDevOpsDescriptor,
    #[facet(rename = "displayName")]
    pub display_name: String,
    #[facet(rename = "id")]
    pub id: AzureDevOpsUserId,
    #[facet(rename = "imageUrl")]
    pub image_url: String,
    #[facet(rename = "uniqueName")]
    pub unique_name: String,
    #[facet(rename = "url")]
    pub url: String,
}
