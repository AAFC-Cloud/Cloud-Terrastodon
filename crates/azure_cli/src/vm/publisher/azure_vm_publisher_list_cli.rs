use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureComputePublishersListRequest;
use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List VM publishers for a subscription and location.
#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherListArgs {
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
}

impl AzureVmPublisherListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        // Resolve the subscription argument (string) into a SubscriptionId.
        let subscription = {
            let s = self.subscription;
            let s = s.trim();
            // First, try to parse as a subscription id (uuid or /subscriptions/<uuid>).
            match s.parse() {
                Ok(id) => id,
                Err(_) => {
                    // Try to match by subscription name (case-insensitive).
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

        info!(subscription = %subscription, location = %location, "Fetching VM publishers");
        let publishers = AzureComputePublishersListRequest {
            authentication,
            subscription_id: (subscription),
            location: (location),
        }
        .execute(executor)
        .await?;

        Ok(CliOutput::facet_with_text(
            publishers,
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
