use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

use crate::scope::AsScope;
use crate::scope::Scope;

pub type ManagementGroupId = String;

/// `az account management-group list --no-register --output json`
/// ```
/// {
///   "displayName": "OPS",
///   "id": "/providers/Microsoft.Management/managementGroups/55555555-5555-5555-5555-555555555555",
///   "name": "55555555-5555-5555-5555-555555555555",  
///   "tenantId": "66666666-6666-6666-6666-666666666666",
///   "type": "Microsoft.Management/managementGroups"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ManagementGroup {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: ManagementGroupId,
    pub name: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "type")]
    pub kind: String,
}
impl std::fmt::Display for ManagementGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(&self.name)?;
        f.write_str(")")?;
        Ok(())
    }
}
impl AsScope for ManagementGroup {
    fn as_scope(&self) -> Scope {
        Scope::ManagementGroup {
            name: self.name.clone(),
        }
    }
}

pub async fn fetch_management_groups() -> Result<Vec<ManagementGroup>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.use_cache_dir(Some(PathBuf::from("ignore/management_groups")));
    cmd.args([
        "account",
        "management-group",
        "list",
        "--no-register",
        "--output",
        "json",
    ]);
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_management_groups().await?;
        println!("Found {} management groups:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
