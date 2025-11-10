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
    cmd.args(&["rest", "--method", "GET", "--url", url]);
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
