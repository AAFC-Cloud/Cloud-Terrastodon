use arbitrary::Arbitrary;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// Account licensing vocabulary used by the [7.1 access-level contract](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#accountlicensetype).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsAccountLicenseType {
    None,
    EarlyAdopter,
    Express,
    Professional,
    Advanced,
    Stakeholder,
    /// An unmodeled response value, preserved in release builds.
    #[arbitrary(skip)]
    Other(String),
}

impl AzureDevOpsAccountLicenseType {
    /// Return the REST spelling without altering an unmodeled response value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::EarlyAdopter => "earlyAdopter",
            Self::Express => "express",
            Self::Professional => "professional",
            Self::Advanced => "advanced",
            Self::Stakeholder => "stakeholder",
            Self::Other(value) => value,
        }
    }
}

/// User-supplied selections must name a modeled REST value.
impl FromStr for AzureDevOpsAccountLicenseType {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "none" => Self::None,
            "earlyAdopter" => Self::EarlyAdopter,
            "express" => Self::Express,
            "professional" => Self::Professional,
            "advanced" => Self::Advanced,
            "stakeholder" => Self::Stakeholder,
            _ => {
                eyre::bail!("Unknown Azure DevOps account license type: {value:?}");
            }
        })
    }
}

impl From<&AzureDevOpsAccountLicenseType> for String {
    fn from(value: &AzureDevOpsAccountLicenseType) -> Self {
        value.as_str().to_owned()
    }
}

/// Response decoding preserves future platform values in release builds.
impl TryFrom<String> for AzureDevOpsAccountLicenseType {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse() {
            Ok(value) => Ok(value),
            Err(_) => {
                #[cfg(debug_assertions)]
                unreachable!(
                    "Unknown {} response value: {value:?}",
                    std::any::type_name::<Self>()
                );
                #[cfg(not(debug_assertions))]
                Ok(Self::Other(value))
            }
        }
    }
}

impl Display for AzureDevOpsAccountLicenseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAccountLicenseType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAccountLicenseType);
