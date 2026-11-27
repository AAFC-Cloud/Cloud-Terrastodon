use arbitrary::Arbitrary;
use facet::Facet;

/// The [operation result](https://learn.microsoft.com/en-us/graph/api/resources/directoryaudit?view=graph-rest-1.0#properties)
/// of a Microsoft Graph v1.0 directory audit event.
#[derive(Arbitrary, Clone, Debug, Eq, Facet, PartialEq)]
#[facet(proxy = String)]
#[repr(C)]
pub enum EntraDirectoryAuditResult {
    Success,
    Failure,
    Timeout,
    UnknownFutureValue,
    #[arbitrary(skip)]
    Unknown(String),
}

impl EntraDirectoryAuditResult {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Timeout => "timeout",
            Self::UnknownFutureValue => "unknownFutureValue",
            Self::Unknown(value) => value,
        }
    }
}

impl From<String> for EntraDirectoryAuditResult {
    fn from(value: String) -> Self {
        match value.as_str() {
            "success" => Self::Success,
            "failure" => Self::Failure,
            "timeout" => Self::Timeout,
            "unknownFutureValue" => Self::UnknownFutureValue,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&EntraDirectoryAuditResult> for String {
    fn from(value: &EntraDirectoryAuditResult) -> Self {
        value.as_str().to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryAuditResult);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryAuditResult);
