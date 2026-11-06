#![allow(
    clippy::disallowed_types,
    reason = "Provider-defined Terraform values need raw JSON; importing azure_types::ArbitraryJson would create a cycle. The module scope includes generated Facet codecs."
)]

use crate::version::SemVer;
use facet::Facet;
use facet_json::RawJson;
use std::collections::BTreeMap;

#[derive(Debug, Facet)]
pub struct TerraformPlan {
    pub format_version: SemVer,
    pub terraform_version: SemVer,
    pub planned_values: TerraformPlanPlannedValues,
    pub resource_changes: Vec<TerraformPlanResourceChange>,
}

#[derive(Debug, Facet)]
pub struct TerraformPlanPlannedValues {
    pub root_module: TerraformPlanPlannedValuesRootModule,
}

#[derive(Debug, Facet)]
pub struct TerraformPlanPlannedValuesRootModule {
    pub resources: Vec<TerraformPlanPlannedValuesResource>,
}

#[derive(Debug, Facet)]
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

#[derive(Debug, Facet)]
#[facet(rename_all = "kebab-case")]
#[repr(C)]
pub enum TerraformPlanResourceMode {
    Managed,
}

#[derive(Debug, Facet)]
pub struct TerraformPlanResourceChange {
    pub address: String,
    pub mode: TerraformPlanResourceMode,
    #[facet(rename = "type")]
    pub r#type: String,
    pub name: String,
    pub provider_name: String,
    pub change: TerraformPlanResourceChangeChange,
}

#[derive(Debug, Facet)]
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
#[derive(Debug, Facet, Eq, PartialEq)]
#[facet(rename_all = "kebab-case")]
#[repr(C)]
pub enum TerraformChangeAction {
    NoOp,
    Create,
    Read,
    Update,
    Delete,
}
