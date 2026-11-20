use cloud_terrastodon_azure_types::UserPrincipalName;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;

/// The nonblank `user.name` from an [Azure CLI profile](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/_profile.py).
/// Preserve its spelling because the session compares the recorded and current names.
/// The name is private account display data and is redacted in `Debug` output.
#[derive(Clone, Eq, PartialEq, Facet)]
#[facet(proxy = String)]
pub struct AzureCliAccountName(String);

impl AzureCliAccountName {
    /// Borrow the name for display or validation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AzureCliAccountName {
    type Error = Report;

    fn try_from(name: String) -> Result<Self> {
        ensure!(
            !name.trim().is_empty(),
            "Azure CLI session has no account identity"
        );
        Ok(Self(name))
    }
}

impl From<&AzureCliAccountName> for String {
    fn from(name: &AzureCliAccountName) -> Self {
        name.0.clone()
    }
}

impl TryFrom<AzureCliAccountName> for UserPrincipalName {
    type Error = Report;

    /// Interpret a CLI profile name using the user principal name syntax rules.
    fn try_from(name: AzureCliAccountName) -> Result<Self> {
        Self::try_new(name.0)
    }
}

impl fmt::Debug for AzureCliAccountName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AzureCliAccountName([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_names_and_preserves_profile_spelling() -> Result<()> {
        for blank in ["", " \t"] {
            assert!(AzureCliAccountName::try_from(blank.to_owned()).is_err());
        }
        let profile_name = "Synthetic-Account";
        let name = AzureCliAccountName::try_from(profile_name.to_owned())?;
        assert_eq!(name.as_str(), profile_name);
        assert!(!format!("{name:?}").contains(profile_name));
        Ok(())
    }
}
