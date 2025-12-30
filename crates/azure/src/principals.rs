use crate::prelude::fetch_all_security_groups;
use crate::prelude::fetch_all_service_principals;
use crate::prelude::fetch_all_users;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_azure_types::prelude::PrincipalCollection;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use itertools::Itertools;
use std::future::IntoFuture;
use std::path::PathBuf;
use tokio::try_join;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct PrincipalListRequest;

pub fn fetch_all_principals() -> PrincipalListRequest {
    PrincipalListRequest
}

#[async_trait]
impl CacheableCommand for PrincipalListRequest {
    type Output = PrincipalCollection;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "principals"]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching principals (users, security groups, and service principals)");
        let (users, security_groups, service_principals) = try_join!(
            fetch_all_users().into_future(),
            fetch_all_security_groups(),
            fetch_all_service_principals()
        )?;
        let principals: Vec<Principal> = users
            .into_iter()
            .map_into()
            .chain(security_groups.into_iter().map_into())
            .chain(service_principals.into_iter().map_into())
            .collect();
        debug!("Found {} principals", principals.len());
        Ok(PrincipalCollection::new(principals))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PrincipalListRequest);

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
