use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsLicenseEntitlement;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_azure_devops_license_entitlements() -> eyre::Result<Vec<AzureDevOpsLicenseEntitlement>> {
    debug!("Fetching Azure DevOps user entitlements");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "invoke"]);
    cmd.args(["--area", "licensing"]);
    cmd.args(["--resource", "entitlements"]);
    cmd.args(["--api-version", "7.2-preview"]);
    cmd.args(["--encoding", "utf-8"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "licensing",
            "entitlements",
        ]),
        valid_for: Duration::from_hours(1),
    });
    
    #[derive(Deserialize)]
    struct InvokeResponse {
        continuation_token: Option<Value>,
        count: u32,
        value: Vec<AzureDevOpsLicenseEntitlement>,
    }
    
    let resp = cmd.run::<InvokeResponse>().await?;
    let entitlements = resp.value;
    
    debug!(
        "Found {} Azure DevOps user entitlements",
        resp.count
    );
    
    if resp.continuation_token.is_some() {
        todo!("Add support for continuation token...");
    }
    
    Ok(entitlements)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let entitlements = fetch_azure_devops_license_entitlements().await?;
        println!("Found {} user entitlements", entitlements.len());
          for entitlement in entitlements.iter().take(5) {
            println!(
                "User: {} ({}) - License: {:?} - Status: {:?}",
                entitlement.user.display_name,
                entitlement.user.unique_name,
                entitlement.license,
                entitlement.status
            );
        }
        
        Ok(())
    }
}
