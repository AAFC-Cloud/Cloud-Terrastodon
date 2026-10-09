use arbitrary::Arbitrary;
use std::str::FromStr;

/// The named days or day combinations used by a build schedule.
///
/// Microsoft's [Go SDK wire model](https://github.com/microsoft/azure-devops-go-api/blob/dev/azuredevops/build/models.go#L1643)
/// represents ScheduleDays as text, unlike the TypeScript SDK's numeric enum.
/// Named values are recognized. Unmodeled combinations and future values panic
/// in debug builds and retain their exact wire spelling in release builds.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildScheduleDays {
    None,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
    All,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureDevOpsBuildScheduleDays {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Monday => "monday",
            Self::Tuesday => "tuesday",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
            Self::Saturday => "saturday",
            Self::Sunday => "sunday",
            Self::All => "all",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildScheduleDays {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "none" => Self::None,
            "monday" => Self::Monday,
            "tuesday" => Self::Tuesday,
            "wednesday" => Self::Wednesday,
            "thursday" => Self::Thursday,
            "friday" => Self::Friday,
            "saturday" => Self::Saturday,
            "sunday" => Self::Sunday,
            "all" => Self::All,
            value => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value.to_owned())
            }
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildScheduleDays {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildScheduleDays> for String {
    fn from(value: &AzureDevOpsBuildScheduleDays) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildScheduleDays {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildScheduleDays);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildScheduleDays);
