use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::ServicePrincipal;
use cloud_terrastodon_command::CacheBehaviour;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_all_service_principals() -> eyre::Result<Vec<ServicePrincipal>> {
    debug!("Fetching service principals");
    let query = MicrosoftGraphHelper::new(
        "https://graph.microsoft.com/v1.0/servicePrincipals",
        CacheBehaviour::Some {
            path: PathBuf::from("service_principals"),
            valid_for: Duration::from_hours(8),
        },
    );
    let entries: Vec<ServicePrincipal> = query.fetch_all().await?;
    debug!("Found {} service principals", entries.len());
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_service_principals;
    use cloud_terrastodon_azure_types::prelude::ServicePrincipal;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found: Vec<ServicePrincipal> = fetch_all_service_principals().await?;
        println!("Found {} service principals", found.len());
        assert!(found.len() > 10);
        Ok(())
    }
}
