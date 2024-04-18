use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use crate::prelude::ManagementGroupId;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyDefinition {
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: String,
    pub metadata: HashMap<String, Value>,
    pub mode: String,
    pub name: String,
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "policyRule")]
    pub policy_rule: serde_json::Value,
    #[serde(rename = "policyType")]
    pub policy_type: String,
    #[serde(rename = "systemData")]
    pub system_data: Value,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for PolicyDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}

pub async fn fetch_policy_definitions(
    management_group: Option<ManagementGroupId>,
    subscription: Option<String>,
) -> Result<Vec<PolicyDefinition>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "definition", "list", "--output", "json"]);
    if let Some(management_group) = management_group {
        cmd.args(["--management-group", &management_group]);
    }
    if let Some(subscription) = subscription {
        cmd.args(["--subscription", &subscription]);
    }
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_definitions(None, None).await?;
        println!("Found {} policy definitions:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
