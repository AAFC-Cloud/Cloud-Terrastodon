use crate::prelude::build_arm_rest_get_command;
use cloud_terrastodon_azure_types::prelude::Location;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

pub struct LocationListRequest {
    pub subscription_id: SubscriptionId,
}

pub fn fetch_all_locations(subscription_id: SubscriptionId) -> LocationListRequest {
    LocationListRequest { subscription_id }
}

#[async_trait]
impl CacheableCommand for LocationListRequest {
    type Output = Vec<Location>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "account",
            "list-locations",
            "--subscription",
            &self.subscription_id.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://management.azure.com/subscriptions/{}/locations?api-version=2022-12-01",
            self.subscription_id
        );
        let cmd = build_arm_rest_get_command(&url, self.cache_key());

        #[derive(serde::Deserialize)]
        struct Response {
            value: Vec<Location>,
        }
        let rtn = cmd.run::<Response>().await?;
        Ok(rtn.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(LocationListRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_locations;
    use crate::prelude::fetch_all_subscriptions;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let subs = fetch_all_subscriptions(tenant_id).await?;
        let mut found_unrecognized_location = false;
        for sub in subs {
            let locations = fetch_all_locations(sub.id).await?;
            found_unrecognized_location |= locations
                .iter()
                .any(|location| location.name.as_other().is_some());
        }
        if found_unrecognized_location {
            eyre::bail!("One or more subscriptions had unrecognized locations");
        }
        Ok(())
    }
}
