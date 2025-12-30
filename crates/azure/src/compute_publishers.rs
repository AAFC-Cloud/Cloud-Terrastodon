use cloud_terrastodon_azure_types::prelude::ComputePublisherId;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

pub struct ComputePublishersListRequest {
    subscription_id: SubscriptionId,
    location: LocationName,
}

pub fn fetch_compute_publishers(
    subscription_id: SubscriptionId,
    location: LocationName,
) -> ComputePublishersListRequest {
    ComputePublishersListRequest {
        subscription_id,
        location,
    }
}

#[async_trait]
impl CacheableCommand for ComputePublishersListRequest {
    type Output = Vec<ComputePublisherId>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "vm",
            "list-publishers",
            self.subscription_id.to_string().as_str(),
            self.location.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = format!(
            "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers?api-version=2024-11-01",
            subscription_id = self.subscription_id,
            location = self.location
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", &url]);
        cmd.cache(self.cache_key());
        #[derive(serde::Deserialize)]
        struct Row {
            id: ComputePublisherId,
            // The location and name are also present but are contained within the ID so we ignore them.
        }
        let rtn = cmd
            .run::<Vec<Row>>()
            .await?
            .into_iter()
            .map(|row| row.id)
            .collect();
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ComputePublishersListRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_subscriptions;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let subs = fetch_all_subscriptions().await?;
        let sub = subs.first().unwrap();
        let publishers =
            crate::prelude::fetch_compute_publishers(sub.id, LocationName::CanadaCentral).await?;
        assert!(!publishers.is_empty());
        println!("Found {} VM publishers", publishers.len());
        println!("{publishers:#?}");
        Ok(())
    }
}
