use clap::Args;
use cloud_terrastodon_azure::prelude::ComputePublisherId;
use cloud_terrastodon_azure::prelude::LocationName;
use cloud_terrastodon_azure::prelude::SubscriptionId;
use cloud_terrastodon_azure::prelude::fetch_all_locations;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure::prelude::fetch_compute_publishers;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Interactively pick subscriptions, locations and publishers.
#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherBrowseArgs {}

impl AzureVmPublisherBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        // 1) Pick subscriptions
        info!("Fetching subscriptions");
        let subs = fetch_all_subscriptions().await?;
        let chosen_subs = PickerTui::new(subs)
            .set_header("Select one or more subscriptions (Tab to mark multiple)")
            .pick_many()?;

        // 2) For each chosen subscription fetch locations and present them as choices
        let mut location_choices: Vec<Choice<(SubscriptionId, LocationName)>> = Vec::new();
        for sub in &chosen_subs {
            info!(subscription_id = %sub.id, subscription_name = %sub.name, "Fetching locations for subscription");
            let locations = fetch_all_locations(&sub.id).await?;
            for loc in locations {
                let key = format!("{} - {} ({})", sub.name, loc.display_name, loc.name);
                location_choices.push(Choice {
                    key,
                    value: (sub.id, loc.name),
                });
            }
        }

        let chosen_locations = PickerTui::new(location_choices)
            .set_header("Select one or more locations (Tab to mark multiple)")
            .pick_many()?;

        // 3) Fetch publishers for each (subscription, location) and accumulate unique publishers
        use std::collections::HashSet;
        let mut publisher_set: HashSet<ComputePublisherId> = HashSet::new();
        for (sub_id, loc_name) in &chosen_locations {
            let pubs = fetch_compute_publishers(sub_id, loc_name).await?;
            for p in pubs {
                publisher_set.insert(p);
            }
        }

        let mut publisher_choices: Vec<ComputePublisherId> = publisher_set.into_iter().collect();
        // Sort for deterministic order
        publisher_choices.sort();

        let chosen_publishers = PickerTui::new(publisher_choices)
            .set_header("Select one or more publishers (Tab to mark multiple)")
            .pick_many()?;

        // Print selected publisher ids
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for p in chosen_publishers {
            writeln!(out, "{}", p)?;
        }

        Ok(())
    }
}
