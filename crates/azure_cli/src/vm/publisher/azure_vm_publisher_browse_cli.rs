use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureComputePublisherImageOfferListRequest;
use cloud_terrastodon_azure::AzureComputePublisherImageOfferSkuListRequest;
use cloud_terrastodon_azure::AzureComputePublisherImageOfferSkuVersionListRequest;
use cloud_terrastodon_azure::AzureComputePublishersListRequest;
use cloud_terrastodon_azure::AzureLocationListRequest;
use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::ComputePublisherId;
use cloud_terrastodon_azure::ComputePublisherVmImageOfferId;
use cloud_terrastodon_azure::ComputePublisherVmImageOfferSkuId;
use cloud_terrastodon_azure::ComputePublisherVmImageOfferSkuVersionId;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// Interactively pick subscriptions, locations and publishers.
#[derive(Facet, Debug, Clone)]
pub struct AzureVmPublisherBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureVmPublisherBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Decision {
            Print,
            Continue,
        }
        // 1) Pick subscriptions
        info!("Fetching subscriptions");
        let authentication = self.authentication.resolve(executor).await?;
        let subs = AzureSubscriptionListRequest { authentication }
            .execute(executor)
            .await?;
        let (chosen_subs, maybe_error) = PickerTui::<_>::new()
            .set_header("Select one or more subscriptions (Tab to mark multiple)")
            .pick_many(subs, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;
        maybe_error?;

        // 2) For each chosen subscription fetch locations and present them as choices
        let mut location_choices: Vec<Choice<(SubscriptionId, AzureLocationName)>> = Vec::new();
        for sub in &chosen_subs {
            info!(subscription_id = %sub.id, subscription_name = %sub.name, "Fetching locations for subscription");
            let locations = AzureLocationListRequest {
                authentication,
                subscription_id: (sub.id),
            }
            .execute(executor)
            .await?;
            for loc in locations {
                let key = format!("{} - {} ({})", sub.name, loc.display_name, loc.name);
                location_choices.push(Choice {
                    key,
                    value: (sub.id, loc.name.clone()),
                });
            }
        }

        let (chosen_locations, maybe_error) = PickerTui::<_>::new()
            .set_header("Select one or more locations (Tab to mark multiple)")
            .pick_many(location_choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;
        maybe_error?;

        // 3) Fetch publishers for each (subscription, location) and accumulate unique publishers
        use std::collections::HashSet;
        let mut publisher_set: HashSet<ComputePublisherId> = HashSet::new();
        for (sub_id, loc_name) in &chosen_locations {
            info!(subscription_id = %sub_id, location = %loc_name, "Fetching publishers for subscription and location");
            let pubs = AzureComputePublishersListRequest {
                authentication,
                subscription_id: (*sub_id),
                location: (loc_name.clone()),
            }
            .execute(executor)
            .await?;
            for p in pubs {
                publisher_set.insert(p);
            }
        }

        let mut publisher_choices: Vec<ComputePublisherId> = publisher_set.into_iter().collect();
        publisher_choices.sort();

        let (chosen_publishers, maybe_error) = PickerTui::<_>::new()
            .set_header("Select one or more publishers (Tab to mark multiple)")
            .pick_many(publisher_choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;
        maybe_error?;

        // 4) Decide to print or continue diving
        let decision = PickerTui::<_>::new()
            .set_header("Publishers: print or continue?")
            .pick_one(
                [
                    Choice {
                        key: "Print selected publishers".into(),
                        value: Decision::Print,
                    },
                    Choice {
                        key: "Continue to offers".into(),
                        value: Decision::Continue,
                    },
                ],
                executor.cancellation(),
            )
            .await?;

        if decision == Decision::Print {
            return Ok(CliOutput::facet_with_text(
                chosen_publishers,
                |values, writer, cancellation| {
                    for value in values {
                        cancellation.bail_if_cancelled()?;
                        writeln!(writer, "{value}")?;
                    }
                    Ok(())
                },
            ));
        }

        // 5) For each chosen publisher fetch offers and accumulate unique offers
        let mut offer_set: HashSet<ComputePublisherVmImageOfferId> = HashSet::new();
        for p in &chosen_publishers {
            info!(subscription_id = %p.subscription_id, location = %p.location_name, publisher = %p.publisher_name, "Fetching offers for publisher");
            let offers = AzureComputePublisherImageOfferListRequest {
                authentication,
                subscription_id: (p.subscription_id),
                location: (p.location_name.clone()),
                publisher_name: (p.publisher_name.clone()),
            }
            .execute(executor)
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

        let (chosen_offers, maybe_error): (Vec<ComputePublisherVmImageOfferId>, _) =
            PickerTui::<_>::new()
                .set_header("Select one or more offers (Tab to mark multiple)")
                .pick_many(offer_display_choices, executor.cancellation())
                .await
                .into_chosen_and_maybe_error()?;
        maybe_error?;

        // 6) Decide to print or continue diving
        let decision = PickerTui::<_>::new()
            .set_header("Offers: print or continue?")
            .pick_one(
                [
                    Choice {
                        key: "Print selected offers".into(),
                        value: Decision::Print,
                    },
                    Choice {
                        key: "Continue to SKUs".into(),
                        value: Decision::Continue,
                    },
                ],
                executor.cancellation(),
            )
            .await?;

        if decision == Decision::Print {
            return Ok(CliOutput::facet_with_text(
                chosen_offers,
                |values, writer, cancellation| {
                    for value in values {
                        cancellation.bail_if_cancelled()?;
                        writeln!(writer, "{value}")?;
                    }
                    Ok(())
                },
            ));
        }

        // 7) For each chosen offer fetch SKUs and accumulate unique SKUs
        let mut sku_set: HashSet<ComputePublisherVmImageOfferSkuId> = HashSet::new();
        for o in &chosen_offers {
            info!(subscription_id = %o.subscription_id, location = %o.location_name, publisher = %o.publisher_name, offer = %o.offer_name, "Fetching SKUs for offer");
            let skus = AzureComputePublisherImageOfferSkuListRequest {
                authentication,
                subscription_id: (o.subscription_id),
                location: (o.location_name.clone()),
                publisher_name: (o.publisher_name.clone()),
                offer_name: (o.offer_name.clone()),
            }
            .execute(executor)
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

        let (chosen_skus, maybe_error): (Vec<ComputePublisherVmImageOfferSkuId>, _) =
            PickerTui::<_>::new()
                .set_header("Select one or more SKUs (Tab to mark multiple)")
                .pick_many(sku_display_choices, executor.cancellation())
                .await
                .into_chosen_and_maybe_error()?;
        maybe_error?;

        // 8) Decide to print or continue diving
        let decision = PickerTui::<_>::new()
            .set_header("SKUs: print or continue?")
            .pick_one(
                [
                    Choice {
                        key: "Print selected SKUs".into(),
                        value: Decision::Print,
                    },
                    Choice {
                        key: "Continue to versions".into(),
                        value: Decision::Continue,
                    },
                ],
                executor.cancellation(),
            )
            .await?;

        if decision == Decision::Print {
            return Ok(CliOutput::facet_with_text(
                chosen_skus,
                |values, writer, cancellation| {
                    for value in values {
                        cancellation.bail_if_cancelled()?;
                        writeln!(writer, "{value}")?;
                    }
                    Ok(())
                },
            ));
        }

        // 9) Display version infos (fetch and print version ids for chosen SKUs)
        let mut version_ids: Vec<ComputePublisherVmImageOfferSkuVersionId> = Vec::new();
        for s in &chosen_skus {
            info!(subscription_id = %s.subscription_id, location = %s.location_name, publisher = %s.publisher_name, offer = %s.offer_name, sku = %s.sku_name, "Fetching versions for SKU");
            let versions = AzureComputePublisherImageOfferSkuVersionListRequest {
                authentication,
                subscription_id: (s.subscription_id),
                location: (s.location_name.clone()),
                publisher_name: (s.publisher_name.clone()),
                offer_name: (s.offer_name.clone()),
                sku_name: (s.sku_name.clone()),
            }
            .execute(executor)
            .await?;
            version_ids.extend(versions);
        }
        version_ids.sort();

        Ok(CliOutput::facet_with_text(
            version_ids,
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
