use arbitrary::Arbitrary;
use std::str::FromStr;

/// The visibility vocabulary in a shallow project reference.
///
/// See Microsoft's [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#projectvisibility).
/// Unmodeled values panic during debug parsing so new service values are reviewed.
/// Release parsing preserves their exact wire spelling in [`Self::Unknown`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsProjectReferenceVisibility {
    Private,
    Public,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureDevOpsProjectReferenceVisibility {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Private => "private",
            Self::Public => "public",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsProjectReferenceVisibility {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "private" => Self::Private,
            "public" => Self::Public,
            value => {
                #[cfg(debug_assertions)]
                unreachable!(
                    "Unmodeled {} value: {value:?}",
                    std::any::type_name::<Self>()
                );
                #[cfg(not(debug_assertions))]
                Self::Unknown(value.to_owned())
            }
        })
    }
}

impl TryFrom<String> for AzureDevOpsProjectReferenceVisibility {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsProjectReferenceVisibility> for String {
    fn from(value: &AzureDevOpsProjectReferenceVisibility) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsProjectReferenceVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectReferenceVisibility);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectReferenceVisibility);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_project_reference_visibility_parses_known_values() {
        for (wire, expected) in [
            ("private", AzureDevOpsProjectReferenceVisibility::Private),
            ("public", AzureDevOpsProjectReferenceVisibility::Public),
        ] {
            let value: AzureDevOpsProjectReferenceVisibility = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
