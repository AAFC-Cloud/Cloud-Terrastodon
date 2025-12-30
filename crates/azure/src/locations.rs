use cloud_terrastodon_azure_types::prelude::Location;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::{CacheKey, CacheableCommand};
use cloud_terrastodon_command::impl_cacheable_into_future;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;

pub struct LocationListRequest {
    subscription_id: SubscriptionId,
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
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", &url]);
        let key = self.cache_key();
        cmd.cache(key);

        #[derive(serde::Deserialize)]
        struct Response {
            value: Vec<Location>,
        }
        let rtn = cmd.run::<Response>().await?;
        Ok(rtn.value)
    }
}

impl_cacheable_into_future!(LocationListRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_locations;
    use crate::prelude::fetch_all_subscriptions;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let subs = fetch_all_subscriptions().await?;
        let mut fail = false;
        for sub in subs {
            println!("Checking locations for subscription {}", sub.name);
            let locations = fetch_all_locations(sub.id).await?;
            for other in locations.iter().filter_map(|loc| loc.name.as_other()) {
                println!(
                    "Subscription {} has unrecognized location {other:?}",
                    sub.name
                );
                fail = true;
            }
        }
        if fail {
            eyre::bail!("One or more subscriptions had unrecognized locations");
        }
        Ok(())
    }
}
