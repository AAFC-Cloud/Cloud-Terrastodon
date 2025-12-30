use cloud_terrastodon_azure_types::prelude::Location;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_locations(subscription_id: &SubscriptionId) -> eyre::Result<Vec<Location>> {
    let url = format!(
        "https://management.azure.com/subscriptions/{subscription_id}/locations?api-version=2022-12-01"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_behaviour(Some(CacheKey {
        path: PathBuf::from_iter([
            "az",
            "account",
            "list-locations",
            "--subscription",
            &subscription_id.to_string(),
        ]),
        valid_for: Duration::MAX,
    }));
    #[derive(serde::Deserialize)]
    struct Response {
        value: Vec<Location>,
    }
    let rtn = cmd.run::<Response>().await?;
    Ok(rtn.value)
}

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
            let locations = fetch_all_locations(&sub.id).await?;
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
