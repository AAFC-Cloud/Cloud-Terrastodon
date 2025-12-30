use cloud_terrastodon_azure_types::prelude::ComputePublisherId;
use cloud_terrastodon_azure_types::prelude::LocationName;
use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;

pub async fn fetch_compute_publishers(
    subscription_id: &SubscriptionId,
    location: &LocationName,
) -> eyre::Result<Vec<ComputePublisherId>> {
    let url = format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/{location}/publishers?api-version=2024-11-01"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_behaviour(Some(CacheKey::new(PathBuf::from_iter([
        "az",
        "vm",
        "list-publishers",
        subscription_id.to_string().as_str(),
        location.to_string().as_str(),
    ]))));
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

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_subscriptions;
    use cloud_terrastodon_azure_types::prelude::LocationName;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let subs = fetch_all_subscriptions().await?;
        let sub = subs.first().unwrap();
        let publishers =
            crate::prelude::fetch_compute_publishers(&sub.id, &LocationName::CanadaCentral).await?;
        assert!(!publishers.is_empty());
        println!("Found {} VM publishers", publishers.len());
        println!("{publishers:#?}");
        Ok(())
    }
}
