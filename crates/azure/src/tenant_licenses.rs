use cloud_terrastodon_azure_types::prelude::TenantLicense;
use cloud_terrastodon_azure_types::prelude::TenantLicenseCollection;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_tenant_licenses() -> eyre::Result<TenantLicenseCollection> {
    let url = "https://graph.microsoft.com/v1.0/subscribedSkus";
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", url]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "rest", "GET", "subscribedSkus"]),
        valid_for: Duration::from_hours(8),
    });

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        #[expect(dead_code)]
        #[serde(rename = "@odata.context")]
        context: String,
        value: Vec<TenantLicense>,
    }
    let resp = cmd.run::<Response>().await?;
    Ok(TenantLicenseCollection(resp.value))
}

#[cfg(test)]
pub mod test_helpers {
    use crate::prelude::fetch_all_tenant_licenses;
    use cloud_terrastodon_command::CommandOutput;
    use cloud_terrastodon_command::bstr::ByteSlice;
    use eyre::bail;

    /// If the given result is a failure, it MUST contain an AAD Premium P2 license error.
    /// If it is an error ,we also validate that the tenant licenses do not contain the AAD Premium P2 license.
    /// If it is a success, we validate that the tenant licenses DO contain the AAD Premium P2 license.
    pub async fn expect_aad_premium_p2_license<T>(
        result: eyre::Result<T>,
    ) -> eyre::Result<Option<T>> {
        let tenant_licenses = fetch_all_tenant_licenses().await?;
        let has_aad_premium_p2_license = tenant_licenses.has_aad_premium_p2();
        match result {
            Ok(x) => {
                eyre::ensure!(
                    has_aad_premium_p2_license,
                    "Expected AAD Premium P2 license, but it was not found in tenant licenses: {tenant_licenses:#?}"
                );
                Ok(Some(x))
            }
            Err(e) => {
                eyre::ensure!(
                    !has_aad_premium_p2_license,
                    "Expected no AAD Premium P2 license, but it was found in tenant licenses: {tenant_licenses:#?}"
                );
                let Some(command_output) = e.downcast_ref::<CommandOutput>() else {
                    bail!("Expected error to be a CommandOutput, but it was: {e:#}");
                };
                if !command_output.stderr.contains_str("The tenant needs to have Microsoft Entra ID P2 or Microsoft Entra ID Governance license.") {
                    bail!("Expected error to contain AAD Premium P2 license error, but it was: {e:#}");
                }
                eprintln!(
                    "Command failed with expected error due to missing AAD_PREMIUM_P2 licenses."
                );
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_tenant_licenses;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant_licenses = fetch_all_tenant_licenses().await?;
        println!("Tenant licenses: {tenant_licenses:#?}");
        println!(
            "Has AAD_PREMIUM_P2: {}",
            tenant_licenses.has_aad_premium_p2()
        );
        Ok(())
    }
}
