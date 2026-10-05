use arbitrary::Arbitrary;
use std::str::FromStr;

/// Whether a definition document is a definition or a draft.
///
/// Microsoft's [response vocabulary](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#definitionquality) is recognized exactly.
/// Unknown values retain their original wire spelling for forward compatibility.
/// Request filters remain a separate closed vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildDefinitionQuality {
    Definition,
    Draft,
    Unknown(String),
}

impl AzureDevOpsBuildDefinitionQuality {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Definition => "definition",
            Self::Draft => "draft",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildDefinitionQuality {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "definition" => Self::Definition,
            "draft" => Self::Draft,
            value => Self::Unknown(value.to_owned()),
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildDefinitionQuality {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildDefinitionQuality> for String {
    fn from(value: &AzureDevOpsBuildDefinitionQuality) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionQuality);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionQuality);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_definition_quality_preserves_known_and_unknown_wire_values() {
        for (wire, expected) in [
            ("definition", AzureDevOpsBuildDefinitionQuality::Definition),
            ("draft", AzureDevOpsBuildDefinitionQuality::Draft),
            (
                "futureSyntheticValue",
                AzureDevOpsBuildDefinitionQuality::Unknown("futureSyntheticValue".to_owned()),
            ),
            (
                "DEFINITION",
                AzureDevOpsBuildDefinitionQuality::Unknown("DEFINITION".to_owned()),
            ),
        ] {
            let value: AzureDevOpsBuildDefinitionQuality = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
