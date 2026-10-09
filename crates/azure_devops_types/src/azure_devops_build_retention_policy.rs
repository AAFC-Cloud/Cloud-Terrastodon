use arbitrary::Arbitrary;

/// Retention rules stored on a build definition.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#retentionpolicy).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildRetentionPolicy {
    pub artifacts: Option<Vec<String>>,
    pub artifact_types_to_delete: Option<Vec<String>>,
    pub branches: Option<Vec<String>>,
    pub days_to_keep: Option<i32>,
    pub delete_build_record: Option<bool>,
    pub delete_test_results: Option<bool>,
    pub minimum_to_keep: Option<i32>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildRetentionPolicy);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildRetentionPolicy);
