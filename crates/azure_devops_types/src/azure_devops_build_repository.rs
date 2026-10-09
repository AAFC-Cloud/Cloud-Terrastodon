use crate::AzureDevOpsBuildRepositoryId;
use crate::AzureDevOpsBuildRepositoryName;
use crate::AzureDevOpsBuildRepositoryType;
use crate::AzureDevOpsBuildRepositoryUrl;
use arbitrary::Arbitrary;
use std::collections::BTreeMap;

/// A source-provider repository used by a build definition.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildrepository).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildRepository {
    pub id: AzureDevOpsBuildRepositoryId,
    pub name: AzureDevOpsBuildRepositoryName,
    pub r#type: AzureDevOpsBuildRepositoryType,
    pub url: AzureDevOpsBuildRepositoryUrl,
    /// Provider-specific branch spelling; absent when the provider has no branch.
    pub default_branch: Option<String>,
    pub checkout_submodules: Option<bool>,
    /// The API represents this setting as text rather than a boolean.
    pub clean: Option<String>,
    pub root_folder: Option<String>,
    /// An open dictionary of provider-specific settings.
    pub properties: Option<BTreeMap<String, String>>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildRepository);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildRepository);
