use arbitrary::Arbitrary;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsLicenseType {
    /// Express means "Basic" in the UI
    AccountExpress,

    AccountStakeholder,

    /// Basic+Test plans
    AccountAdvanced,

    MsdnEligible,

    MsdnEnterprise,

    MsdnProfessional,

    None,

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

impl FromStr for AzureDevOpsLicenseType {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Account-Express" | "Basic" | "basic" | "express" => Self::AccountExpress,
            "Account-Stakeholder" | "Stakeholder" | "stakeholder" => Self::AccountStakeholder,
            "Account-Advanced" | "Advanced" | "advanced" | "test" | "Test" => Self::AccountAdvanced,
            "Msdn-Eligible" => Self::MsdnEligible,
            "Msdn-Enterprise" => Self::MsdnEnterprise,
            "Msdn-Professional" => Self::MsdnProfessional,
            "None" | "none" => Self::None,
            other => Self::Other(other.to_owned()),
        })
    }
}

impl From<&AzureDevOpsLicenseType> for String {
    fn from(value: &AzureDevOpsLicenseType) -> Self {
        match value {
            AzureDevOpsLicenseType::AccountExpress => "Account-Express".to_owned(),
            AzureDevOpsLicenseType::AccountStakeholder => "Account-Stakeholder".to_owned(),
            AzureDevOpsLicenseType::AccountAdvanced => "Account-Advanced".to_owned(),
            AzureDevOpsLicenseType::MsdnEligible => "Msdn-Eligible".to_owned(),
            AzureDevOpsLicenseType::MsdnEnterprise => "Msdn-Enterprise".to_owned(),
            AzureDevOpsLicenseType::MsdnProfessional => "Msdn-Professional".to_owned(),
            AzureDevOpsLicenseType::None => "None".to_owned(),
            AzureDevOpsLicenseType::Other(value) => value.clone(),
        }
    }
}

impl TryFrom<String> for AzureDevOpsLicenseType {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsLicenseType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsLicenseType);

#[cfg(test)]
mod license_tests {
    use super::AzureDevOpsLicenseType;

    #[test]
    pub fn deserializes_account_express() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Account-Express""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::AccountExpress);
        Ok(())
    }

    #[test]
    pub fn deserializes_account_stakeholder() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Account-Stakeholder""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::AccountStakeholder);
        Ok(())
    }

    #[test]
    pub fn deserializes_account_advanced() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Account-Advanced""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::AccountAdvanced);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_eligible() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Msdn-Eligible""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::MsdnEligible);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_enterprise() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Msdn-Enterprise""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::MsdnEnterprise);
        Ok(())
    }

    #[test]
    pub fn deserializes_msdn_professional() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Msdn-Professional""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::MsdnProfessional);
        Ok(())
    }

    #[test]
    pub fn deserializes_none() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""None""#)?;
        assert_eq!(license, AzureDevOpsLicenseType::None);
        Ok(())
    }

    #[test]
    pub fn deserializes_other() -> eyre::Result<()> {
        let license = facet_json::from_str::<AzureDevOpsLicenseType>(r#""Custom-License""#)?;
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
