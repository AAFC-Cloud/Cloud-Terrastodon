use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureComputePublisherImageOfferSkuVersionListRequest;
use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::ComputePublisherName;
use cloud_terrastodon_azure::ComputePublisherVmImageOfferName;
use cloud_terrastodon_azure::ComputePublisherVmImageOfferSkuName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List versions for a publisher's offer SKU for a subscription and location.
#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherOfferSkuVersionListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Subscription id or name to query. If an id is provided it will be parsed, otherwise
    /// the list of subscriptions will be searched for a subscription with a matching name.
    /// This scope is explicit and independent of the selected account.
    #[facet(figue::named)]
    pub subscription: String,

    /// Location to query. Defaults to `canadacentral`.
    #[facet(figue::named)]
    pub location: Option<AzureLocationName>,

    /// Publisher name to query (e.g. center-for-internet-security-inc).
    #[facet(figue::named)]
    pub publisher: String,

    /// Offer name to query.
    #[facet(figue::named)]
    pub offer: String,

    /// SKU name to query.
    #[facet(figue::named)]
    pub sku: String,
}

impl AzureVmPublisherOfferSkuVersionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        // Resolve the subscription argument (string) into a SubscriptionId.
        let subscription = {
            let s = self.subscription;
            let s = s.trim();
            match s.parse() {
                Ok(id) => id,
                Err(_) => {
                    let subs = AzureSubscriptionListRequest { authentication }
                        .execute(executor)
                        .await?;
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
        };

        let location = self.location.unwrap_or(AzureLocationName::CanadaCentral);

        let publisher = self.publisher.parse::<ComputePublisherName>()?;
        let offer = self.offer.parse::<ComputePublisherVmImageOfferName>()?;
        let sku = self.sku.parse::<ComputePublisherVmImageOfferSkuName>()?;

        info!(subscription = %subscription, location = %location, publisher = %publisher, offer = %offer, sku = %sku, "Fetching VM publisher offer SKU versions");
        let versions = AzureComputePublisherImageOfferSkuVersionListRequest {
            authentication,
            subscription_id: (subscription),
            location: (location),
            publisher_name: (publisher),
            offer_name: (offer),
            sku_name: (sku),
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet_with_text(
            versions,
            |values, writer, cancellation| {
                for value in values {
                    cancellation.bail_if_cancelled()?;
                    writeln!(writer, "{value}")?;
                }
                Ok(())
            },
        ))
    }
}
