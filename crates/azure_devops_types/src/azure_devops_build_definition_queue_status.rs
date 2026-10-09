use arbitrary::Arbitrary;
use std::str::FromStr;

/// Whether a definition permits builds to be queued and started.
///
/// Microsoft's [response vocabulary](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#definitionqueuestatus) is recognized exactly.
/// Unknown values panic in debug builds so unmodeled vocabulary is surfaced.
/// Release builds retain their original wire spelling for forward compatibility.
/// Request filters remain a separate closed vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildDefinitionQueueStatus {
    Enabled,
    Paused,
    Disabled,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureDevOpsBuildDefinitionQueueStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Enabled => "enabled",
            Self::Paused => "paused",
            Self::Disabled => "disabled",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildDefinitionQueueStatus {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "enabled" => Self::Enabled,
            "paused" => Self::Paused,
            "disabled" => Self::Disabled,
            value => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value.to_owned())
            }
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildDefinitionQueueStatus {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildDefinitionQueueStatus> for String {
    fn from(value: &AzureDevOpsBuildDefinitionQueueStatus) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionQueueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionQueueStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionQueueStatus);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_definition_queue_status_preserves_known_wire_values() {
        for (wire, expected) in [
            ("enabled", AzureDevOpsBuildDefinitionQueueStatus::Enabled),
            ("paused", AzureDevOpsBuildDefinitionQueueStatus::Paused),
            ("disabled", AzureDevOpsBuildDefinitionQueueStatus::Disabled),
        ] {
            let value: AzureDevOpsBuildDefinitionQueueStatus = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
