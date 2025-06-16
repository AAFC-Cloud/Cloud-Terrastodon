use crate::prelude::fetch_all_security_groups;
use crate::prelude::fetch_all_service_principals;
use crate::prelude::fetch_all_users;
use cloud_terrastodon_azure_types::prelude::Principal;
use itertools::Itertools;
use tokio::try_join;
use tracing::info;

pub async fn fetch_all_principals() -> eyre::Result<Vec<Principal>> {
    info!("Fetching principals (users, security groups, and service principals)");
    let (users, security_groups, service_principals) = try_join!(
        fetch_all_users(),
        fetch_all_security_groups(),
        fetch_all_service_principals()
    )?;
    let principals: Vec<Principal> = users
        .into_iter()
        .map_into()
        .chain(security_groups.into_iter().map_into())
        .chain(service_principals.into_iter().map_into())
        .collect();
    info!("Found {} principals", principals.len());
    Ok(principals)
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_principals;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let found = fetch_all_principals().await?;
        println!("Found {} principals", found.len());
        assert!(found.len() > 10);
        Ok(())
    }
}
