use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTestPlan {
    pub area: Option<AzureDevOpsTestPlanShallowReference>,
    pub build: Option<AzureDevOpsTestPlanShallowReference>,
    pub build_definition: Option<AzureDevOpsTestPlanShallowReference>,
    pub description: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub id: u32,
    pub iteration: Option<String>,
    pub name: String,
    pub owner: Option<AzureDevOpsTestPlanIdentityRef>,
    pub previous_build: Option<AzureDevOpsTestPlanShallowReference>,
    pub project: Option<AzureDevOpsTestPlanShallowReference>,
    pub release_environment_definition:
        Option<AzureDevOpsTestPlanReleaseEnvironmentDefinitionReference>,
    pub revision: Option<u32>,
    pub root_suite: Option<AzureDevOpsTestPlanShallowReference>,
    pub start_date: Option<DateTime<Utc>>,
    pub state: Option<String>,
    pub test_outcome_settings: Option<AzureDevOpsTestPlanTestOutcomeSettings>,
    pub updated_by: Option<AzureDevOpsTestPlanIdentityRef>,
    pub updated_date: Option<DateTime<Utc>>,
    pub url: Option<String>,
    pub client_url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct AzureDevOpsTestPlanShallowReference {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTestPlanIdentityRef {
    #[facet(rename = "_links")]
    pub links: Option<ArbitraryJson>,
    pub descriptor: Option<String>,
    pub directory_alias: Option<String>,
    pub display_name: Option<String>,
    pub id: Option<String>,
    pub image_url: Option<String>,
    pub inactive: Option<bool>,
    pub is_aad_identity: Option<bool>,
    pub is_container: Option<bool>,
    pub is_deleted_in_origin: Option<bool>,
    pub profile_url: Option<String>,
    pub unique_name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTestPlanReleaseEnvironmentDefinitionReference {
    pub definition_id: Option<u32>,
    pub environment_definition_id: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTestPlanTestOutcomeSettings {
    pub sync_outcome_across_suites: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTestPlan);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestPlan);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsTestPlan>);
