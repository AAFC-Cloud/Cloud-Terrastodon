use arbitrary::Arbitrary;
use std::str::FromStr;

/// The trigger vocabulary used by build definitions.
///
/// Microsoft documentation: [DefinitionTriggerType](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#definitiontriggertype).
/// Unknown values panic in debug builds and preserve their original spelling in
/// release builds for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsBuildDefinitionTriggerType {
    None,
    ContinuousIntegration,
    BatchedContinuousIntegration,
    Schedule,
    GatedCheckIn,
    BatchedGatedCheckIn,
    PullRequest,
    BuildCompletion,
    All,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureDevOpsBuildDefinitionTriggerType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::ContinuousIntegration => "continuousIntegration",
            Self::BatchedContinuousIntegration => "batchedContinuousIntegration",
            Self::Schedule => "schedule",
            Self::GatedCheckIn => "gatedCheckIn",
            Self::BatchedGatedCheckIn => "batchedGatedCheckIn",
            Self::PullRequest => "pullRequest",
            Self::BuildCompletion => "buildCompletion",
            Self::All => "all",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsBuildDefinitionTriggerType {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "none" => Self::None,
            "continuousIntegration" => Self::ContinuousIntegration,
            "batchedContinuousIntegration" => Self::BatchedContinuousIntegration,
            "schedule" => Self::Schedule,
            "gatedCheckIn" => Self::GatedCheckIn,
            "batchedGatedCheckIn" => Self::BatchedGatedCheckIn,
            "pullRequest" => Self::PullRequest,
            "buildCompletion" => Self::BuildCompletion,
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

impl TryFrom<String> for AzureDevOpsBuildDefinitionTriggerType {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildDefinitionTriggerType> for String {
    fn from(value: &AzureDevOpsBuildDefinitionTriggerType) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionTriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionTriggerType);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionTriggerType);
