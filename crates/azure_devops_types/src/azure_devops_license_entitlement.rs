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
    /// Express means "Basic" in the UI
    #[serde(rename = "Account-Express")]
    AccountExpress,
    #[serde(rename = "Account-Stakeholder")]
    AccountStakeholder,
    /// Basic+Test plans
    #[serde(rename = "Account-Advanced")]
    AccountAdvanced,
    #[serde(rename = "Msdn-Eligible")]
    MsdnEligible,
    #[serde(rename = "Msdn-Enterprise")]
    MsdnEnterprise,
    #[serde(rename = "Msdn-Professional")]
    MsdnProfessional,
    #[serde(untagged)]
    Other(String),
}
impl AzureDevOpsLicenseEntitlementLicense {
    /// https://azure.microsoft.com/en-us/pricing/details/devops/azure-devops-services/
    pub fn cost_per_month_cad(&self) -> f64 {
        match self {
            AzureDevOpsLicenseEntitlementLicense::AccountExpress => 8.30,
            AzureDevOpsLicenseEntitlementLicense::AccountStakeholder => 0.0,
            AzureDevOpsLicenseEntitlementLicense::AccountAdvanced => 71.93,
            AzureDevOpsLicenseEntitlementLicense::MsdnEnterprise => 0.00,
            AzureDevOpsLicenseEntitlementLicense::MsdnProfessional => 0.00,
            AzureDevOpsLicenseEntitlementLicense::MsdnEligible => 0.00,
            AzureDevOpsLicenseEntitlementLicense::Other(_) => 0.0,
        }
    }
}

#[cfg(test)]
mod license_tests {
    use super::AzureDevOpsLicenseEntitlementLicense;

    #[test]
    pub fn deserializes_account_express() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Account-Express""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::AccountExpress);
        Ok(())
    }

    #[test]
    pub fn deserializes_account_stakeholder() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Account-Stakeholder""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::AccountStakeholder);
        Ok(())
    }

    #[test]
    pub fn deserializes_account_advanced() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Account-Advanced""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::AccountAdvanced);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_eligible() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Msdn-Eligible""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::MsdnEligible);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_enterprise() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Msdn-Enterprise""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::MsdnEnterprise);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_professional() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Msdn-Professional""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::MsdnProfessional);
        Ok(())
    }

    #[test]
    pub fn deserializes_other() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseEntitlementLicense>(r#""Custom-License""#)?;
        assert_eq!(license, AzureDevOpsLicenseEntitlementLicense::Other("Custom-License".to_string()));
        Ok(())
    }
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
    pub last_accessed_date: LastAccessedDate,
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
