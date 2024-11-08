use cloud_terrastodon_core_azure_types::prelude::Principal;
use itertools::Itertools;
use tokio::try_join;
use tracing::info;

use crate::prelude::fetch_all_security_groups;
use crate::prelude::fetch_all_service_principals;
use crate::prelude::fetch_all_users;

pub async fn fetch_all_principals() -> anyhow::Result<Vec<Principal>> {
    info!("Fetching principals (users, security groups, and service principals)");
    let (users, security_groups, service_principals) = try_join!(
        fetch_all_users(),
        fetch_all_security_groups(),
        fetch_all_service_principals()
    )?;
    Ok(users
        .into_iter()
        .map_into()
        .chain(security_groups.into_iter().map_into())
        .chain(service_principals.into_iter().map_into())
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_principals;

    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> {
        let found = fetch_all_principals().await?;
        println!("Found {} principals", found.len());
        assert!(found.len() > 10);
        Ok(())
    }
}
