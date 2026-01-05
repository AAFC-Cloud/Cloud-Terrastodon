use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;


#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestPlan {
    pub area: Option<ShallowReference>,
    pub build: Option<ShallowReference>,
    #[serde(rename = "buildDefinition")]
    pub build_definition: Option<ShallowReference>,
    pub description: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
    pub id: u32,
    pub iteration: Option<String>,
    pub name: String,
    pub owner: Option<AzureDevOpsTestPlanIdentityRef>,
    #[serde(rename = "previousBuild")]
    pub previous_build: Option<ShallowReference>,
    pub project: Option<ShallowReference>,
    #[serde(rename = "releaseEnvironmentDefinition")]
    pub release_environment_definition: Option<AzureDevOpsTestPlanReleaseEnvironmentDefinitionReference>,
    pub revision: Option<u32>,
    #[serde(rename = "rootSuite")]
    pub root_suite: Option<ShallowReference>,
    #[serde(rename = "startDate")]
    pub start_date: Option<DateTime<Utc>>,
    pub state: Option<String>,
    #[serde(rename = "testOutcomeSettings")]
    pub test_outcome_settings: Option<AzureDevOpsTestPlanTestOutcomeSettings>,
    #[serde(rename = "updatedBy")]
    pub updated_by: Option<AzureDevOpsTestPlanIdentityRef>,
    #[serde(rename = "updatedDate")]
    pub updated_date: Option<DateTime<Utc>>,
    pub url: Option<String>,
    #[serde(rename = "clientUrl")]
    pub client_url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct ShallowReference {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestPlanIdentityRef {
    #[serde(rename = "_links")]
    pub links: Option<Value>,
    pub descriptor: Option<String>,
    #[serde(rename = "directoryAlias")]
    pub directory_alias: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    pub inactive: Option<bool>,
    #[serde(rename = "isAadIdentity")]
    pub is_aad_identity: Option<bool>,
    #[serde(rename = "isContainer")]
    pub is_container: Option<bool>,
    #[serde(rename = "isDeletedInOrigin")]
    pub is_deleted_in_origin: Option<bool>,
    #[serde(rename = "profileUrl")]
    pub profile_url: Option<String>,
    #[serde(rename = "uniqueName")]
    pub unique_name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestPlanReleaseEnvironmentDefinitionReference {
    #[serde(rename = "definitionId")]
    pub definition_id: Option<u32>,
    #[serde(rename = "environmentDefinitionId")]
    pub environment_definition_id: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTestPlanTestOutcomeSettings {
    #[serde(rename = "syncOutcomeAcrossSuites")]
    pub sync_outcome_across_suites: Option<bool>,
}