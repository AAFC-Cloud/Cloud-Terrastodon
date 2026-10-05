use arbitrary::Arbitrary;
use std::str::FromStr;

/// The open visibility vocabulary in a shallow project reference.
///
/// See Microsoft's [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#projectvisibility).
/// Unknown values preserve their exact wire spelling.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsProjectReferenceVisibility {
    Private,
    Public,
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
            value => Self::Unknown(value.to_owned()),
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
    fn build_project_reference_visibility_preserves_known_and_unknown_values() {
        for (wire, expected) in [
            ("private", AzureDevOpsProjectReferenceVisibility::Private),
            ("public", AzureDevOpsProjectReferenceVisibility::Public),
            (
                "futureSyntheticValue",
                AzureDevOpsProjectReferenceVisibility::Unknown("futureSyntheticValue".to_owned()),
            ),
        ] {
            let value: AzureDevOpsProjectReferenceVisibility = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
