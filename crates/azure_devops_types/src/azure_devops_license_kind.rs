use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum AzureDevOpsLicenseKind {
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

impl AzureDevOpsLicenseKind {
    /// https://azure.microsoft.com/en-us/pricing/details/devops/azure-devops-services/
    pub fn cost_per_month_cad(&self) -> f64 {
        match self {
            AzureDevOpsLicenseKind::AccountExpress => 8.30,
            AzureDevOpsLicenseKind::AccountStakeholder => 0.0,
            AzureDevOpsLicenseKind::AccountAdvanced => 71.93,
            AzureDevOpsLicenseKind::MsdnEnterprise => 0.00,
            AzureDevOpsLicenseKind::MsdnProfessional => 0.00,
            AzureDevOpsLicenseKind::MsdnEligible => 0.00,
            AzureDevOpsLicenseKind::Other(_) => 0.0,
        }
    }
}

impl std::str::FromStr for AzureDevOpsLicenseKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use serde to leverage existing deserialization logic for the enum
        let q = format!("\"{}\"", s);
        let license = serde_json::from_str::<AzureDevOpsLicenseKind>(&q)?;
        Ok(license)
    }
}

#[cfg(test)]
mod license_tests {
    use super::AzureDevOpsLicenseKind;

    #[test]
    pub fn deserializes_account_express() -> eyre::Result<()> {
        let license =
            serde_json::from_str::<AzureDevOpsLicenseKind>(r#""Account-Express""#)?;
        assert_eq!(
            license,
            AzureDevOpsLicenseKind::AccountExpress
        );
        Ok(())
    }

    #[test]
    pub fn deserializes_account_stakeholder() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseKind>(
            r#""Account-Stakeholder""#,
        )?;
        assert_eq!(
            license,
            AzureDevOpsLicenseKind::AccountStakeholder
        );
        Ok(())
    }

    #[test]
    pub fn deserializes_account_advanced() -> eyre::Result<()> {
        let license =
            serde_json::from_str::<AzureDevOpsLicenseKind>(r#""Account-Advanced""#)?;
        assert_eq!(
            license,
            AzureDevOpsLicenseKind::AccountAdvanced
        );
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_eligible() -> eyre::Result<()> {
        let license =
            serde_json::from_str::<AzureDevOpsLicenseKind>(r#""Msdn-Eligible""#)?;
        assert_eq!(license, AzureDevOpsLicenseKind::MsdnEligible);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_enterprise() -> eyre::Result<()> {
        let license =
            serde_json::from_str::<AzureDevOpsLicenseKind>(r#""Msdn-Enterprise""#)?;
        assert_eq!(
            license,
            AzureDevOpsLicenseKind::MsdnEnterprise
        );
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_professional() -> eyre::Result<()> {
        let license =
            serde_json::from_str::<AzureDevOpsLicenseKind>(r#""Msdn-Professional""#)?;
        assert_eq!(
            license,
            AzureDevOpsLicenseKind::MsdnProfessional
        );
        Ok(())
    }

    #[test]
    pub fn deserializes_other() -> eyre::Result<()> {
        let license =
            serde_json::from_str::<AzureDevOpsLicenseKind>(r#""Custom-License""#)?;
        assert_eq!(
            license,
            AzureDevOpsLicenseKind::Other("Custom-License".to_string())
        );
        Ok(())
    }
}