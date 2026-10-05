use arbitrary::Arbitrary;
use std::str::FromStr;

/// The response type of a build definition.
///
/// Microsoft's [response vocabulary](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#definitiontype) is recognized exactly.
/// Unknown values retain their original wire spelling for forward compatibility.
/// Request filters remain a separate closed vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildDefinitionType {
    Xaml,
    Build,
    Unknown(String),
}

impl AzureDevOpsBuildDefinitionType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Xaml => "xaml",
            Self::Build => "build",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildDefinitionType {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "xaml" => Self::Xaml,
            "build" => Self::Build,
            value => Self::Unknown(value.to_owned()),
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildDefinitionType {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildDefinitionType> for String {
    fn from(value: &AzureDevOpsBuildDefinitionType) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionType);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_definition_type_preserves_known_and_unknown_wire_values() {
        for (wire, expected) in [
            ("xaml", AzureDevOpsBuildDefinitionType::Xaml),
            ("build", AzureDevOpsBuildDefinitionType::Build),
            (
                "futureSyntheticValue",
                AzureDevOpsBuildDefinitionType::Unknown("futureSyntheticValue".to_owned()),
            ),
            (
                "XAML",
                AzureDevOpsBuildDefinitionType::Unknown("XAML".to_owned()),
            ),
        ] {
            let value: AzureDevOpsBuildDefinitionType = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
