use arbitrary::Arbitrary;
use eyre::Result;

/// Why a fresh inventory prevents pruning a previously planned folder.
/// The string proxy preserves the human-readable reason in prune JSON reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(u8)]
pub enum AzureDevOpsBuildFolderPruneSkipReason {
    NoLongerExists,
    PathChanged,
    ContainsDefinitions,
    HasChildFolders,
}

impl AzureDevOpsBuildFolderPruneSkipReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoLongerExists => "Folder no longer exists",
            Self::PathChanged => "Folder path changed since planning",
            Self::ContainsDefinitions => "Folder now contains build definitions",
            Self::HasChildFolders => "Folder still has child folders",
        }
    }
}

impl std::fmt::Display for AzureDevOpsBuildFolderPruneSkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&AzureDevOpsBuildFolderPruneSkipReason> for String {
    fn from(value: &AzureDevOpsBuildFolderPruneSkipReason) -> Self {
        value.as_str().to_owned()
    }
}

impl TryFrom<String> for AzureDevOpsBuildFolderPruneSkipReason {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "Folder no longer exists" => Ok(Self::NoLongerExists),
            "Folder path changed since planning" => Ok(Self::PathChanged),
            "Folder now contains build definitions" => Ok(Self::ContainsDefinitions),
            "Folder still has child folders" => Ok(Self::HasChildFolders),
            _ => eyre::bail!("Unknown build folder pruning skip reason"),
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderPruneSkipReason);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderPruneSkipReason);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_folder_prune_skip_reason_preserves_its_message() {
        let reason = AzureDevOpsBuildFolderPruneSkipReason::HasChildFolders;
        let message = "Folder still has child folders";
        assert_eq!(reason.as_str(), message);
        assert_eq!(
            AzureDevOpsBuildFolderPruneSkipReason::try_from(message.to_owned()).unwrap(),
            reason
        );
        assert!(AzureDevOpsBuildFolderPruneSkipReason::try_from("unknown".to_owned()).is_err());
    }
}
