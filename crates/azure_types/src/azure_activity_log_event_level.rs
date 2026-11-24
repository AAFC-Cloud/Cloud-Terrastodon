use arbitrary::Arbitrary;
use facet::Facet;

/// The severity of an Azure Monitor 2015-04-01 [activity log event](https://learn.microsoft.com/en-us/rest/api/monitor/activity-logs/list?view=rest-monitor-2015-04-01#eventlevel).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureActivityLogEventLevel {
    Critical,
    Error,
    Warning,
    Informational,
    Verbose,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureActivityLogEventLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Critical => "Critical",
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Informational => "Informational",
            Self::Verbose => "Verbose",
            Self::Unknown(value) => value,
        }
    }
}

impl From<String> for AzureActivityLogEventLevel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Critical" => Self::Critical,
            "Error" => Self::Error,
            "Warning" => Self::Warning,
            "Informational" => Self::Informational,
            "Verbose" => Self::Verbose,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureActivityLogEventLevel> for String {
    fn from(value: &AzureActivityLogEventLevel) -> Self {
        value.as_str().to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureActivityLogEventLevel);
cloud_terrastodon_registry::register_arbitrary!(AzureActivityLogEventLevel);
