use arbitrary::Arbitrary;
use std::str::FromStr;

/// The project-state vocabulary in a shallow project reference.
///
/// See Microsoft's [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#projectstate).
/// Unmodeled values panic during debug parsing so new service values are reviewed.
/// Release parsing preserves their exact wire spelling in [`Self::Unknown`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsProjectReferenceState {
    Deleting,
    New,
    WellFormed,
    CreatePending,
    All,
    Unchanged,
    Deleted,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureDevOpsProjectReferenceState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Deleting => "deleting",
            Self::New => "new",
            Self::WellFormed => "wellFormed",
            Self::CreatePending => "createPending",
            Self::All => "all",
            Self::Unchanged => "unchanged",
            Self::Deleted => "deleted",
            Self::Unknown(value) => value,
        }
    }
}

impl FromStr for AzureDevOpsProjectReferenceState {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "deleting" => Self::Deleting,
            "new" => Self::New,
            "wellFormed" => Self::WellFormed,
            "createPending" => Self::CreatePending,
            "all" => Self::All,
            "unchanged" => Self::Unchanged,
            "deleted" => Self::Deleted,
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

impl TryFrom<String> for AzureDevOpsProjectReferenceState {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsProjectReferenceState> for String {
    fn from(value: &AzureDevOpsProjectReferenceState) -> Self {
        value.as_str().to_owned()
    }
}

impl std::fmt::Display for AzureDevOpsProjectReferenceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectReferenceState);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectReferenceState);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_project_reference_state_parses_known_values() {
        for (wire, expected) in [
            ("deleting", AzureDevOpsProjectReferenceState::Deleting),
            ("new", AzureDevOpsProjectReferenceState::New),
            ("wellFormed", AzureDevOpsProjectReferenceState::WellFormed),
            (
                "createPending",
                AzureDevOpsProjectReferenceState::CreatePending,
            ),
            ("all", AzureDevOpsProjectReferenceState::All),
            ("unchanged", AzureDevOpsProjectReferenceState::Unchanged),
            ("deleted", AzureDevOpsProjectReferenceState::Deleted),
        ] {
            let value: AzureDevOpsProjectReferenceState = wire.parse().unwrap();
            assert_eq!(value, expected);
            assert_eq!(value.as_str(), wire);
        }
    }
}
