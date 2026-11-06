use arbitrary::Arbitrary;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// MSDN licensing vocabulary used by the [7.1 access-level contract](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/update-user-entitlement?view=azure-devops-rest-7.1#msdnlicensetype).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsMsdnLicenseType {
    None,
    Eligible,
    Professional,
    Platforms,
    TestProfessional,
    Premium,
    Ultimate,
    Enterprise,
    /// An unmodeled response value, preserved in release builds.
    #[arbitrary(skip)]
    Other(String),
}

impl AzureDevOpsMsdnLicenseType {
    /// Return the REST spelling without altering an unmodeled response value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Eligible => "eligible",
            Self::Professional => "professional",
            Self::Platforms => "platforms",
            Self::TestProfessional => "testProfessional",
            Self::Premium => "premium",
            Self::Ultimate => "ultimate",
            Self::Enterprise => "enterprise",
            Self::Other(value) => value,
        }
    }
}

/// User-supplied selections must name a modeled REST value.
impl FromStr for AzureDevOpsMsdnLicenseType {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "none" => Self::None,
            "eligible" => Self::Eligible,
            "professional" => Self::Professional,
            "platforms" => Self::Platforms,
            "testProfessional" => Self::TestProfessional,
            "premium" => Self::Premium,
            "ultimate" => Self::Ultimate,
            "enterprise" => Self::Enterprise,
            _ => {
                eyre::bail!("Unknown Azure DevOps MSDN license type: {value:?}");
            }
        })
    }
}

impl From<&AzureDevOpsMsdnLicenseType> for String {
    fn from(value: &AzureDevOpsMsdnLicenseType) -> Self {
        value.as_str().to_owned()
    }
}

/// Response decoding preserves future platform values in release builds.
impl TryFrom<String> for AzureDevOpsMsdnLicenseType {
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

impl Display for AzureDevOpsMsdnLicenseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsMsdnLicenseType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsMsdnLicenseType);
