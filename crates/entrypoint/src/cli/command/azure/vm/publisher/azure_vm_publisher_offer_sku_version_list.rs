use clap::Args;
use cloud_terrastodon_azure::prelude::ComputePublisherName;
use cloud_terrastodon_azure::prelude::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure::prelude::ComputePublisherVmImageOfferSkuName;
use cloud_terrastodon_azure::prelude::LocationName;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure::prelude::fetch_compute_publisher_image_offer_sku_versions;
use cloud_terrastodon_azure::prelude::get_active_subscription_id;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List versions for a publisher's offer SKU for a subscription and location.
#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherOfferSkuVersionListArgs {
    /// Subscription id or name to query. If an id is provided it will be parsed, otherwise
    /// the list of subscriptions will be searched for a subscription with a matching name.
    /// Defaults to the active account subscription.
    #[arg(long)]
    pub subscription: Option<String>,

    /// Location to query. Defaults to `canadacentral`.
    #[arg(long)]
    pub location: Option<LocationName>,

    /// Publisher name to query (e.g. center-for-internet-security-inc).
    #[arg(long)]
    pub publisher: String,

    /// Offer name to query.
    #[arg(long)]
    pub offer: String,

    /// SKU name to query.
    #[arg(long)]
    pub sku: String,
}

impl AzureVmPublisherOfferSkuVersionListArgs {
    pub async fn invoke(self) -> Result<()> {
        // Resolve the subscription argument (string) into a SubscriptionId.
        let subscription = match self.subscription {
            Some(s) => {
                let s = s.trim();
                match s.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        let subs = fetch_all_subscriptions().await?;
                        let target = s.to_lowercase();
                        if let Some(found) = subs
                            .into_iter()
                            .find(|sub| sub.name.eq_ignore_ascii_case(&target))
                        {
                            found.id
                        } else {
                            eyre::bail!("No subscription found matching id or name '{s}'");
                        }
                    }
                }
            }
            None => get_active_subscription_id().await?,
        };

        let location = self.location.unwrap_or(LocationName::CanadaCentral);

        let publisher = self.publisher.parse::<ComputePublisherName>()?;
        let offer = self.offer.parse::<ComputePublisherVmImageOfferName>()?;
        let sku = self.sku.parse::<ComputePublisherVmImageOfferSkuName>()?;

        info!(subscription = %subscription, location = %location, publisher = %publisher, offer = %offer, sku = %sku, "Fetching VM publisher offer SKU versions");
        let versions = fetch_compute_publisher_image_offer_sku_versions(&subscription, &location, &publisher, &offer, &sku).await?;

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for v in versions {
            writeln!(out, "{}", v)?;
        }

        Ok(())
    }
}
