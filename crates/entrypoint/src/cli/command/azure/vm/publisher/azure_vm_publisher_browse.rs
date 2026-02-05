use clap::Args;
use cloud_terrastodon_azure::prelude::ComputePublisherId;
use cloud_terrastodon_azure::prelude::ComputePublisherVmImageOfferId;
use cloud_terrastodon_azure::prelude::ComputePublisherVmImageOfferSkuId;
use cloud_terrastodon_azure::prelude::ComputePublisherVmImageOfferSkuVersionId;
use cloud_terrastodon_azure::prelude::LocationName;
use cloud_terrastodon_azure::prelude::SubscriptionId;
use cloud_terrastodon_azure::prelude::fetch_all_locations;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure::prelude::fetch_compute_publisher_image_offer_sku_versions;
use cloud_terrastodon_azure::prelude::fetch_compute_publisher_image_offer_skus;
use cloud_terrastodon_azure::prelude::fetch_compute_publisher_image_offers;
use cloud_terrastodon_azure::prelude::fetch_compute_publishers;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Interactively pick subscriptions, locations and publishers.
#[derive(Args, Debug, Clone)]
pub struct AzureVmPublisherBrowseArgs;

impl AzureVmPublisherBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Decision {
            Print,
            Continue,
        }
        // 1) Pick subscriptions
        info!("Fetching subscriptions");
        let subs = fetch_all_subscriptions().await?;
        let chosen_subs = PickerTui::new()
            .set_header("Select one or more subscriptions (Tab to mark multiple)")
            .pick_many(subs)?;

        // 2) For each chosen subscription fetch locations and present them as choices
        let mut location_choices: Vec<Choice<(SubscriptionId, LocationName)>> = Vec::new();
        for sub in &chosen_subs {
            info!(subscription_id = %sub.id, subscription_name = %sub.name, "Fetching locations for subscription");
            let locations = fetch_all_locations(sub.id).await?;
            for loc in locations {
                let key = format!("{} - {} ({})", sub.name, loc.display_name, loc.name);
                location_choices.push(Choice {
                    key,
                    value: (sub.id, loc.name),
                });
            }
        }

        let chosen_locations = PickerTui::new()
            .set_header("Select one or more locations (Tab to mark multiple)")
            .pick_many(location_choices)?;

        // 3) Fetch publishers for each (subscription, location) and accumulate unique publishers
        use std::collections::HashSet;
        let mut publisher_set: HashSet<ComputePublisherId> = HashSet::new();
        for (sub_id, loc_name) in &chosen_locations {
            info!(subscription_id = %sub_id, location = %loc_name, "Fetching publishers for subscription and location");
            let pubs = fetch_compute_publishers(*sub_id, loc_name.clone()).await?;
            for p in pubs {
                publisher_set.insert(p);
            }
        }

        let mut publisher_choices: Vec<ComputePublisherId> = publisher_set.into_iter().collect();
        publisher_choices.sort();

        let chosen_publishers = PickerTui::new()
            .set_header("Select one or more publishers (Tab to mark multiple)")
            .pick_many(publisher_choices)?;

        // 4) Decide to print or continue diving
        let decision = PickerTui::new()
            .set_header("Publishers: print or continue?")
            .pick_one([
                Choice {
                    key: "Print selected publishers".into(),
                    value: Decision::Print,
                },
                Choice {
                    key: "Continue to offers".into(),
                    value: Decision::Continue,
                },
            ])?;

        if decision == Decision::Print {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for p in chosen_publishers {
                writeln!(out, "{}", p)?;
            }
            return Ok(());
        }

        // 5) For each chosen publisher fetch offers and accumulate unique offers
        let mut offer_set: HashSet<ComputePublisherVmImageOfferId> = HashSet::new();
        for p in &chosen_publishers {
            info!(subscription_id = %p.subscription_id, location = %p.location_name, publisher = %p.publisher_name, "Fetching offers for publisher");
            let offers = fetch_compute_publisher_image_offers(
                p.subscription_id,
                p.location_name.clone(),
                p.publisher_name.clone(),
            )
            .await?;
            for o in offers {
                offer_set.insert(o);
            }
        }

        let mut offer_choices: Vec<ComputePublisherVmImageOfferId> =
            offer_set.into_iter().collect();
        offer_choices.sort();

        let offer_display_choices: Vec<Choice<ComputePublisherVmImageOfferId>> = offer_choices
            .into_iter()
            .map(|o| Choice {
                key: format!(
                    "{} - {} ({})",
                    o.publisher_name, o.offer_name, o.location_name
                ),
                value: o,
            })
            .collect();

        let chosen_offers: Vec<ComputePublisherVmImageOfferId> = PickerTui::new()
            .set_header("Select one or more offers (Tab to mark multiple)")
            .pick_many(offer_display_choices)?;

        // 6) Decide to print or continue diving
        let decision = PickerTui::new()
            .set_header("Offers: print or continue?")
            .pick_one([
                Choice {
                    key: "Print selected offers".into(),
                    value: Decision::Print,
                },
                Choice {
                    key: "Continue to SKUs".into(),
                    value: Decision::Continue,
                },
            ])?;

        if decision == Decision::Print {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for o in chosen_offers {
                writeln!(out, "{}", o)?;
            }
            return Ok(());
        }

        // 7) For each chosen offer fetch SKUs and accumulate unique SKUs
        let mut sku_set: HashSet<ComputePublisherVmImageOfferSkuId> = HashSet::new();
        for o in &chosen_offers {
            info!(subscription_id = %o.subscription_id, location = %o.location_name, publisher = %o.publisher_name, offer = %o.offer_name, "Fetching SKUs for offer");
            let skus = fetch_compute_publisher_image_offer_skus(
                o.subscription_id,
                o.location_name.clone(),
                o.publisher_name.clone(),
                o.offer_name.clone(),
            )
            .await?;
            for s in skus {
                sku_set.insert(s);
            }
        }

        let mut sku_choices: Vec<ComputePublisherVmImageOfferSkuId> = sku_set.into_iter().collect();
        sku_choices.sort();

        let sku_display_choices: Vec<Choice<ComputePublisherVmImageOfferSkuId>> = sku_choices
            .into_iter()
            .map(|s| Choice {
                key: format!(
                    "{} - {} - {} ({})",
                    s.publisher_name, s.offer_name, s.sku_name, s.location_name
                ),
                value: s,
            })
            .collect();

        let chosen_skus: Vec<ComputePublisherVmImageOfferSkuId> = PickerTui::new()
            .set_header("Select one or more SKUs (Tab to mark multiple)")
            .pick_many(sku_display_choices)?;

        // 8) Decide to print or continue diving
        let decision = PickerTui::new()
            .set_header("SKUs: print or continue?")
            .pick_one([
                Choice {
                    key: "Print selected SKUs".into(),
                    value: Decision::Print,
                },
                Choice {
                    key: "Continue to versions".into(),
                    value: Decision::Continue,
                },
            ])?;

        if decision == Decision::Print {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for s in chosen_skus {
                writeln!(out, "{}", s)?;
            }
            return Ok(());
        }

        // 9) Display version infos (fetch and print version ids for chosen SKUs)
        let mut version_ids: Vec<ComputePublisherVmImageOfferSkuVersionId> = Vec::new();
        for s in &chosen_skus {
            info!(subscription_id = %s.subscription_id, location = %s.location_name, publisher = %s.publisher_name, offer = %s.offer_name, sku = %s.sku_name, "Fetching versions for SKU");
            let versions = fetch_compute_publisher_image_offer_sku_versions(
                s.subscription_id,
                s.location_name.clone(),
                s.publisher_name.clone(),
                s.offer_name.clone(),
                s.sku_name.clone(),
            )
            .await?;
            version_ids.extend(versions);
        }
        version_ids.sort();

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for v in version_ids {
            writeln!(out, "{}", v)?;
        }

        Ok(())
    }
}
