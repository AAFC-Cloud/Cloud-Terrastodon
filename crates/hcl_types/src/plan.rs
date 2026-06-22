use crate::version::SemVer;
use facet_json::RawJson;
use std::collections::BTreeMap;

#[derive(Debug, facet::Facet)]
pub struct TerraformPlan {
    pub format_version: SemVer,
    pub terraform_version: SemVer,
    pub planned_values: TerraformPlanPlannedValues,
    pub resource_changes: Vec<TerraformPlanResourceChange>,
}

#[derive(Debug, facet::Facet)]
pub struct TerraformPlanPlannedValues {
    pub root_module: TerraformPlanPlannedValuesRootModule,
}

#[derive(Debug, facet::Facet)]
pub struct TerraformPlanPlannedValuesRootModule {
    pub resources: Vec<TerraformPlanPlannedValuesResource>,
}

#[derive(Debug, facet::Facet)]
pub struct TerraformPlanPlannedValuesResource {
    pub address: String,
    pub mode: TerraformPlanResourceMode,
    #[facet(rename = "type")]
    pub r#type: String,
    pub name: String,
    pub provider_name: String,
    pub schema_version: usize,
    pub values: RawJson<'static>,
    pub sensitive_values: RawJson<'static>,
}

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "kebab-case")]
#[repr(C)]
pub enum TerraformPlanResourceMode {
    Managed,
}

#[derive(Debug, facet::Facet)]
pub struct TerraformPlanResourceChange {
    pub address: String,
    pub mode: TerraformPlanResourceMode,
    #[facet(rename = "type")]
    pub r#type: String,
    pub name: String,
    pub provider_name: String,
    pub change: TerraformPlanResourceChangeChange,
}

#[derive(Debug, facet::Facet)]
pub struct TerraformPlanResourceChangeChange {
    /// <https://developer.hashicorp.com/terraform/internals/json-format#change-representation>
    pub actions: Vec<TerraformChangeAction>,
    pub before: Option<RawJson<'static>>,
    pub after: Option<RawJson<'static>>,
    pub after_unknown: RawJson<'static>,
    /// Can be a bool or a map
    pub before_sensitive: RawJson<'static>,
    pub after_sensitive: BTreeMap<String, RawJson<'static>>,
}

/// <https://developer.hashicorp.com/terraform/internals/json-format#change-representation>
#[derive(Debug, facet::Facet, Eq, PartialEq)]
#[facet(rename_all = "kebab-case")]
#[repr(C)]
pub enum TerraformChangeAction {
    NoOp,
    Create,
    Read,
    Update,
    Delete,
}
