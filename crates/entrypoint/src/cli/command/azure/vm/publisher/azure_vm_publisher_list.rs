use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::LocationName;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_azure::fetch_compute_publishers;
use cloud_terrastodon_azure::get_active_subscription_id;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List VM publishers for a subscription and location.
#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Subscription id or name to query. If an id is provided it will be parsed, otherwise
    /// the list of subscriptions will be searched for a subscription with a matching name.
    /// Defaults to the active account subscription.
    #[arg(long)]
    pub subscription: Option<String>,

    /// Location to query. Defaults to `canadacentral`.
    #[arg(long)]
    pub location: Option<LocationName>,
}

impl AzureVmPublisherListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        // Resolve the subscription argument (string) into a SubscriptionId.
        let subscription = match self.subscription {
            Some(s) => {
                let s = s.trim();
                // First, try to parse as a subscription id (uuid or /subscriptions/<uuid>).
                match s.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        // Try to match by subscription name (case-insensitive).
                        let subs = fetch_all_subscriptions(tenant_id).await?;
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

        info!(subscription = %subscription, location = %location, "Fetching VM publishers");
        let publishers = fetch_compute_publishers(subscription, location).await?;

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for p in publishers {
            // Print the full id of the publisher (e.g. /Subscriptions/.../Publishers/<name>)
            writeln!(out, "{}", p)?;
        }

        Ok(())
    }
}
