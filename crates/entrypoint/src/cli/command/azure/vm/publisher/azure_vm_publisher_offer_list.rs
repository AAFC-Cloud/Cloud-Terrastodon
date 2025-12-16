use clap::Args;
use cloud_terrastodon_azure::prelude::ComputePublisherName;
use cloud_terrastodon_azure::prelude::LocationName;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure::prelude::fetch_compute_publisher_image_offers;
use cloud_terrastodon_azure::prelude::get_active_subscription_id;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List offers from a publisher for a subscription and location.
#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherOfferListArgs {
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
}

impl AzureVmPublisherOfferListArgs {
    pub async fn invoke(self) -> Result<()> {
        // Resolve the subscription argument (string) into a SubscriptionId.
        let subscription = match self.subscription {
            Some(s) => {
                let s = s.trim();
                // First, try to parse as a subscription id (uuid or /subscriptions/<uuid>).
                match s.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        // Try to match by subscription name (case-insensitive).
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

        info!(subscription = %subscription, location = %location, publisher = %publisher, "Fetching VM publisher offers");
        let offers = fetch_compute_publisher_image_offers(&subscription, &location, &publisher).await?;

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for o in offers {
            writeln!(out, "{}", o)?;
        }

        Ok(())
    }
}
