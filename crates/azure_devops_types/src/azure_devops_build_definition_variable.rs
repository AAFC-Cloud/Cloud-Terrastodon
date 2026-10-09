use arbitrary::Arbitrary;

/// A variable stored on a build definition.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinitionvariable).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildDefinitionVariable {
    /// The service omits secret values instead of revealing their contents.
    pub value: Option<String>,
    pub is_secret: Option<bool>,
    pub allow_override: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionVariable);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionVariable);
