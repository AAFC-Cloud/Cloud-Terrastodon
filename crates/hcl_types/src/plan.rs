use crate::version::SemVer;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct TerraformPlan {
    pub format_version: SemVer,
    pub terraform_version: SemVer,
    pub planned_values: TerraformPlanPlannedValues,
    pub resource_changes: Vec<TerraformPlanResourceChange>,
}

#[derive(Debug, Deserialize)]
pub struct TerraformPlanPlannedValues {
    pub root_module: TerraformPlanPlannedValuesRootModule,
}

#[derive(Debug, Deserialize)]
pub struct TerraformPlanPlannedValuesRootModule {
    pub resources: Vec<TerraformPlanPlannedValuesResource>,
}

#[derive(Debug, Deserialize)]
pub struct TerraformPlanPlannedValuesResource {
    pub address: String,
    pub mode: TerraformPlanResourceMode,
    pub r#type: String,
    pub name: String,
    pub provider_name: String,
    pub schema_version: usize,
    pub values: Value,
    pub sensitive_values: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerraformPlanResourceMode {
    Managed,
}

#[derive(Debug, Deserialize)]
pub struct TerraformPlanResourceChange {
    pub address: String,
    pub mode: TerraformPlanResourceMode,
    pub r#type: String,
    pub name: String,
    pub provider_name: String,
    pub change: TerraformPlanResourceChangeChange,
}

#[derive(Debug, Deserialize)]
pub struct TerraformPlanResourceChangeChange {
    pub actions: Vec<TerraformChangeAction>,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub after_unknown: Value,
    /// Can be a bool or a map
    pub before_sensitive: Value,
    pub after_sensitive: serde_json::Map<String, Value>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TerraformChangeAction {
    Create,
    NoOp,
}
