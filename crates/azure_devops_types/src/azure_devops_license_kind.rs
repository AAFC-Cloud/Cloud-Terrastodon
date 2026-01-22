use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum AzureDevOpsLicenseType {
    /// Express means "Basic" in the UI
    #[serde(rename = "Account-Express")]
    #[serde(alias = "Basic")]
    #[serde(alias = "basic")]
    #[serde(alias = "express")]
    AccountExpress,

    #[serde(rename = "Account-Stakeholder")]
    #[serde(alias = "Stakeholder")]
    #[serde(alias = "stakeholder")]
    AccountStakeholder,

    /// Basic+Test plans
    #[serde(rename = "Account-Advanced")]
    #[serde(alias = "Advanced")]
    #[serde(alias = "advanced")]
    #[serde(alias = "test")]
    #[serde(alias = "Test")]
    AccountAdvanced,

    #[serde(rename = "Msdn-Eligible")]
    MsdnEligible,

    #[serde(rename = "Msdn-Enterprise")]
    MsdnEnterprise,

    #[serde(rename = "Msdn-Professional")]
    MsdnProfessional,

    None,

    #[serde(untagged)]
    Other(String),
}

impl AzureDevOpsLicenseType {
    pub const VARIANTS: &'static [AzureDevOpsLicenseType] = &[
        AzureDevOpsLicenseType::AccountExpress,
        AzureDevOpsLicenseType::AccountStakeholder,
        AzureDevOpsLicenseType::AccountAdvanced,
        AzureDevOpsLicenseType::MsdnEligible,
        AzureDevOpsLicenseType::MsdnEnterprise,
        AzureDevOpsLicenseType::MsdnProfessional,
    ];

    /// https://azure.microsoft.com/en-us/pricing/details/devops/azure-devops-services/
    pub fn cost_per_month_cad(&self) -> f64 {
        match self {
            AzureDevOpsLicenseType::AccountExpress => 8.30,
            AzureDevOpsLicenseType::AccountStakeholder => 0.0,
            AzureDevOpsLicenseType::AccountAdvanced => 71.93,
            AzureDevOpsLicenseType::MsdnEnterprise => 0.00,
            AzureDevOpsLicenseType::MsdnProfessional => 0.00,
            AzureDevOpsLicenseType::MsdnEligible => 0.00,
            AzureDevOpsLicenseType::None => 0.00,
            AzureDevOpsLicenseType::Other(_) => 0.0,
        }
    }
}

impl std::str::FromStr for AzureDevOpsLicenseType {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use serde to leverage existing deserialization logic for the enum
        let q = format!("\"{}\"", s);
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(&q)?;
        Ok(license)
    }
}

impl std::fmt::Display for AzureDevOpsLicenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsLicenseType::AccountExpress => write!(f, "Account-Express (Basic)"),
            AzureDevOpsLicenseType::AccountStakeholder => write!(f, "Account-Stakeholder"),
            AzureDevOpsLicenseType::AccountAdvanced => write!(f, "Account-Advanced (Basic+Test)"),
            AzureDevOpsLicenseType::MsdnEligible => write!(f, "Msdn-Eligible"),
            AzureDevOpsLicenseType::MsdnEnterprise => write!(f, "Msdn-Enterprise"),
            AzureDevOpsLicenseType::MsdnProfessional => write!(f, "Msdn-Professional"),
            AzureDevOpsLicenseType::None => write!(f, "None"),
            AzureDevOpsLicenseType::Other(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod license_tests {
    use super::AzureDevOpsLicenseType;

    #[test]
    pub fn deserializes_account_express() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Account-Express""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::AccountExpress);
        Ok(())
    }

    #[test]
    pub fn deserializes_account_stakeholder() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Account-Stakeholder""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::AccountStakeholder);
        Ok(())
    }

    #[test]
    pub fn deserializes_account_advanced() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Account-Advanced""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::AccountAdvanced);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_eligible() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Msdn-Eligible""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::MsdnEligible);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_enterprise() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Msdn-Enterprise""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::MsdnEnterprise);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_professional() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Msdn-Professional""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::MsdnProfessional);
        Ok(())
    }

    #[test]
    pub fn deserializes_none() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""None""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::None);
        Ok(())
    }

    #[test]
    pub fn deserializes_other() -> eyre::Result<()> {
        let license = serde_json::from_str::<AzureDevOpsLicenseType>(r#""Custom-License""#)?;
        assert_eq!(
            license,
            AzureDevOpsLicenseType::Other("Custom-License".to_string())
        );
        Ok(())
    }
    #[test]
    pub fn display_account_express() -> eyre::Result<()> {
        assert_eq!(
            format!("{}", AzureDevOpsLicenseType::AccountExpress),
            "Account-Express (Basic)"
        );
        Ok(())
    }

    #[test]
    pub fn display_account_advanced() -> eyre::Result<()> {
        assert_eq!(
            format!("{}", AzureDevOpsLicenseType::AccountAdvanced),
            "Account-Advanced (Basic+Test)"
        );
        Ok(())
    }
}
