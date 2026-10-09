use arbitrary::Arbitrary;
use std::str::FromStr;

/// The permissions scope used for a definition's jobs.
///
/// Microsoft documentation: [BuildAuthorizationScope](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildauthorizationscope).
/// Unknown values panic in debug builds and preserve their original spelling in
/// release builds for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildAuthorizationScope {
    ProjectCollection,
    Project,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureDevOpsBuildAuthorizationScope {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ProjectCollection => "projectCollection",
            Self::Project => "project",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildAuthorizationScope {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "projectCollection" => Self::ProjectCollection,
            "project" => Self::Project,
            value => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value.to_owned())
            }
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildAuthorizationScope {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildAuthorizationScope> for String {
    fn from(value: &AzureDevOpsBuildAuthorizationScope) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildAuthorizationScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildAuthorizationScope);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildAuthorizationScope);
