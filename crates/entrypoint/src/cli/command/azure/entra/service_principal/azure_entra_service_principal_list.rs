use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_service_principals;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) service principals.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraSpListArgs {}

impl AzureEntraSpListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching service principals");
        let sps = fetch_all_service_principals().await?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for sp in sps {
            writeln!(
                out,
                "{} {:64} {} {}",
                sp.id,
                sp.display_name,
                sp.app_id,
                sp.service_principal_names.join(",")
            )?;
        }
        Ok(())
    }
}
