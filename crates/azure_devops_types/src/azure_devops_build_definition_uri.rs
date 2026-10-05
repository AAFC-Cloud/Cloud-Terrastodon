use crate::AzureDevOpsBuildDefinitionId;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;
use url::Url;

/// An Azure DevOps build definition artifact URI.
///
/// Unlike a definition's REST URL, `vstfs:///Build/Definition/{id}` contains no
/// organization or project. Its definition ID is interpreted within the scope
/// of the containing response or operation.
///
/// See `uri` in Microsoft's [BuildDefinitionReference 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#builddefinitionreference)
/// and the artifact URI in the [Favorites API sample](https://learn.microsoft.com/en-us/rest/api/azure/devops/favorite/favorites/get-favorite-by-artifact?view=azure-devops-rest-7.1#examples).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildDefinitionUri {
    pub definition_id: AzureDevOpsBuildDefinitionId,
}

impl AzureDevOpsBuildDefinitionUri {
    pub const fn new(definition_id: AzureDevOpsBuildDefinitionId) -> Self {
        Self { definition_id }
    }
}

impl FromStr for AzureDevOpsBuildDefinitionUri {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let uri = Url::parse(value)?;
        ensure!(
            uri.scheme() == "vstfs"
                && uri.host().is_none()
                && uri.username().is_empty()
                && uri.password().is_none()
                && uri.port().is_none()
                && uri.query().is_none()
                && uri.fragment().is_none(),
            "Build definition artifact URIs use the vstfs scheme without an authority, query, or fragment"
        );
        let definition_id = uri
            .path()
            .strip_prefix("/Build/Definition/")
            .ok_or_else(|| eyre::eyre!("URI does not identify an Azure DevOps build definition"))?
            .parse()?;
        Ok(Self::new(definition_id))
    }
}

impl From<AzureDevOpsBuildDefinitionId> for AzureDevOpsBuildDefinitionUri {
    fn from(definition_id: AzureDevOpsBuildDefinitionId) -> Self {
        Self::new(definition_id)
    }
}

impl TryFrom<String> for AzureDevOpsBuildDefinitionUri {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<&str> for AzureDevOpsBuildDefinitionUri {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildDefinitionUri> for String {
    fn from(value: &AzureDevOpsBuildDefinitionUri) -> Self {
        value.to_string()
    }
}

impl std::fmt::Display for AzureDevOpsBuildDefinitionUri {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "vstfs:///Build/Definition/{}",
            self.definition_id
        )
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionUri);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionUri);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_artifact_uris_round_trip_their_typed_identity() -> Result<()> {
        for id in [1, 7, i32::MAX] {
            let definition_id = AzureDevOpsBuildDefinitionId::new(id)?;
            let uri = AzureDevOpsBuildDefinitionUri::new(definition_id);
            assert_eq!(
                uri.to_string().parse::<AzureDevOpsBuildDefinitionUri>()?,
                uri
            );
            assert_eq!(uri.definition_id, definition_id);
        }
        assert_eq!(
            "VSTFS:///Build/Definition/7"
                .parse::<AzureDevOpsBuildDefinitionUri>()?
                .to_string(),
            "vstfs:///Build/Definition/7"
        );
        Ok(())
    }

    #[test]
    fn rejects_unrelated_artifacts_and_invalid_definition_identities() {
        for value in [
            "relative",
            "vstfs:///Build/Build/7",
            "https://dev.azure.com/synthetic/project/_apis/build/definitions/7",
            "vstfs://synthetic/Build/Definition/7",
            "vstfs:///Build/Definition/7?revision=2",
            "vstfs:///Build/Definition/7#fragment",
            "vstfs:///Build/Definition/7/revisions",
            "vstfs:///Build/Definition/",
            "vstfs:///Build/Definition/0",
            "vstfs:///Build/Definition/-1",
            "vstfs:///Build/Definition/2147483648",
            "vstfs:///Build/Definition/synthetic",
        ] {
            assert!(value.parse::<AzureDevOpsBuildDefinitionUri>().is_err());
        }
    }
}
