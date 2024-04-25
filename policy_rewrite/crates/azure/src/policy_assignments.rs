use crate::scope::AsScope;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tf::prelude::AzureRMResourceKind;
use tf::prelude::TofuResourceReference;
use std::collections::HashMap;
use std::path::PathBuf;
use tf::prelude::ImportBlock;
use tf::prelude::Sanitizable;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyAssignment {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "enforcementMode")]
    pub enforcement_mode: String,
    pub id: String,
    pub identity: Option<HashMap<String, Value>>,
    pub location: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub name: String,
    #[serde(rename = "nonComplianceMessages")]
    pub non_compliance_messages: Option<Value>,
    #[serde(rename = "notScopes")]
    pub not_scopes: Option<Vec<String>>,
    pub parameters: Option<Value>,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: String,
    pub scope: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicyAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl From<PolicyAssignment> for ImportBlock {
    fn from(policy_assignment: PolicyAssignment) -> Self {
        ImportBlock {
            id: policy_assignment.id.clone(),
            to: TofuResourceReference::AzureRM {
                kind: AzureRMResourceKind::ManagementGroupPolicyAssignment,
                name: policy_assignment.display_name.sanitize(),
            },
        }
    }
}

pub async fn fetch_policy_assignments(
    scope: Option<&impl AsScope>,
    subscription: Option<String>,
) -> Result<Vec<PolicyAssignment>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "policy",
        "assignment",
        "list",
        "--disable-scope-strict-match",
        "--output",
        "json",
    ]);
    let mut cache_key = PathBuf::new();
    cache_key.push("ignore");
    cache_key.push("policy_assignments");
    if let Some(scope) = scope {
        let scope = scope.as_scope();
        cmd.args(["--scope", &scope.expanded_form()]);
        cache_key.push(scope.short_name());
    }
    if let Some(subscription) = subscription {
        cmd.args(["--subscription", &subscription]);
        cache_key.push(subscription)
    }
    cmd.use_cache_dir(Some(cache_key));
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_assignments(None, None).await?;
        println!("Found {} policy assignments:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
