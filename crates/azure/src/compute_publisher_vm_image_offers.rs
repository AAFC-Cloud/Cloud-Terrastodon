use cloud_terrastodon_azure_types::prelude::ComputePublisherName;
use cloud_terrastodon_azure_types::prelude::ComputePublisherVmImageOfferId;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;

pub async fn fetch_compute_publisher_image_offers(
    subscription_id: &SubscriptionId,
    location: &LocationName,
    publisher_name: &ComputePublisherName,
) -> eyre::Result<Vec<ComputePublisherVmImageOfferId>> {
    let url = format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers/{publisher_name}/artifacttypes/vmimage/offers?api-version=2024-07-01"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.cache(CacheKey::new(PathBuf::from_iter([
        "az",
        "vm",
        "list-publishers-offers",
        subscription_id.to_string().as_str(),
        location.to_string().as_str(),
        publisher_name.to_string().as_str(),
    ])));
    #[derive(serde::Deserialize)]
    struct Row {
        id: ComputePublisherVmImageOfferId,
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

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_subscriptions;
    use cloud_terrastodon_azure_types::prelude::ComputePublisherName;
    use cloud_terrastodon_azure_types::prelude::LocationName;
    use cloud_terrastodon_azure_types::prelude::Slug;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let subs = fetch_all_subscriptions().await?;
        let sub = subs.first().unwrap();
        let publisher = ComputePublisherName::try_new("center-for-internet-security-inc")?;
        let offers = super::fetch_compute_publisher_image_offers(
            &sub.id,
            &LocationName::CanadaCentral,
            &publisher,
        )
        .await?;
        for offer in offers.iter() {
            println!("{}", offer);
        }

        Ok(())
    }
}
