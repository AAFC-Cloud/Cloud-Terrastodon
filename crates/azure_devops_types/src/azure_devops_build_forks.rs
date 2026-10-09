use arbitrary::Arbitrary;

/// Pull-request trigger settings for repository forks.
///
/// Microsoft REST documentation: [containing BuildTrigger schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildtrigger).
/// The REST reference documents the base trigger. Fork settings are described by
/// Microsoft's [Forks interface](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/forks)
/// and [TypeScript source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L1893).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildForks {
    pub enabled: Option<bool>,
    pub allow_full_access_token: Option<bool>,
    pub allow_secrets: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildForks);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildForks);
