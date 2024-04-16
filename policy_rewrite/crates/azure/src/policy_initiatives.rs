use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;

use crate::errors::dump_to_ignore_file;
use crate::prelude::ManagementGroupId;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyInitiativePolicyDefinitionGroup {
    #[serde(rename = "additionalMetadataId")]
    pub additional_metadata_id: String,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyInitiativePolicyDefinition {
    #[serde(rename = "groupNames")]
    pub group_names: Vec<String>,
    pub parameters: Value,
    #[serde(rename = "policyDefinitionId")]
    pub policy_definition_id: String,
    #[serde(rename = "policyDefinitionReferenceId")]
    pub policy_definition_reference_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyInitiative {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    pub metadata: HashMap<String, Value>,
    pub name: String,
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "policyDefinitionGroups")]
    pub policy_definition_groups: Option<Vec<PolicyInitiativePolicyDefinitionGroup>>,
    #[serde(rename = "policyDefinitions")]
    pub policy_definitions: Option<Vec<PolicyInitiativePolicyDefinition>>,
    #[serde(rename = "policyType")]
    pub policy_type: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicyInitiative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}

pub async fn fetch_policy_initiatives(
    management_group: Option<ManagementGroupId>,
    subscription: Option<String>,
) -> Result<Vec<PolicyInitiative>> {
    let mut cmd = Command::new("az.cmd");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.args(["policy", "definition", "list", "--output", "json"]);
    if let Some(management_group) = management_group {
        cmd.args(["--management-group", &management_group]);
    }
    if let Some(subscription) = subscription {
        cmd.args(["--subscription", &subscription]);
    }
    let cmd = cmd.spawn()?;
    let output = cmd.wait_with_output().await?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        match serde_json::from_str(&stdout) {
            Ok(results) => Ok(results),
            Err(e) => {
                let context = dump_to_ignore_file(&stdout)?;
                Err(e)
                    .context("deserializing")
                    .context(format!("dumped to {:?}", context))
            }
        }
        // Ok(results)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(Error::msg(error_message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_initiatives(None, None).await?;
        println!("Found {} policy initiatives:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
