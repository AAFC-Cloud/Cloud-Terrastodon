use anyhow::Error;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;
use std::process::Stdio;

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

pub async fn fetch_management_groups() -> Result<Vec<ManagementGroup>> {
    let mut cmd = Command::new("az.cmd");
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.args("account management-group list --no-register --output json".split_whitespace());
    let cmd = cmd.spawn()?;
    let output = cmd.wait_with_output().await?;
    if output.status.success() {
        let response_string = String::from_utf8_lossy(&output.stdout);
        let management_groups: Vec<ManagementGroup> = serde_json::from_str(&response_string)?;
        Ok(management_groups)
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
        let result = fetch_management_groups().await?;
        println!("Found {} management groups:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
