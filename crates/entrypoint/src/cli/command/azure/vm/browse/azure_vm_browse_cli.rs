use crate::cli::azure::vm::browse::AzureVmBrowseOption;
use crate::cli::azure::vm::publisher::AzureVmPublisherBrowseArgs;
use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_azure::prelude::fetch_all_virtual_machines;
use cloud_terrastodon_azure::prelude::fetch_virtual_machine_skus;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use strum::VariantArray;
use tracing::info;

#[derive(Args, Debug, Clone)]
pub struct AzureVmBrowseArgs;

impl AzureVmBrowseArgs {
    pub async fn invoke(self) -> eyre::Result<()> {
        let chosen = PickerTui::new().pick_one(AzureVmBrowseOption::VARIANTS)?;
        match chosen {
            AzureVmBrowseOption::Resources => {
                let chosen_resources = PickerTui::new()
                    .pick_many_reloadable(async |invalidate| {
                        info!("Fetching virtual machines");
                        Ok(fetch_all_virtual_machines()
                            .with_invalidation(invalidate)
                            .await?
                            .into_iter()
                            .map(|vm| Choice {
                                key: format!(
                                    "{vm_name} - {vm_id}",
                                    vm_name = vm.name,
                                    vm_id = vm.id
                                ),
                                value: vm,
                            }))
                    })
                    .await?;
                println!("{}", serde_json::to_string_pretty(&chosen_resources)?);
            }
            AzureVmBrowseOption::Skus => {
                let chosen_subscription = PickerTui::new()
                    .pick_one_reloadable(async |invalidate| {
                        info!("Fetching subscriptions");
                        let subscriptions = fetch_all_subscriptions()
                            .with_invalidation(invalidate)
                            .await?;
                        Ok(subscriptions)
                    })
                    .await?;

                let chosen_skus = PickerTui::new().pick_many_reloadable(async move |invalidate| {
                    info!(%chosen_subscription.name, %chosen_subscription.id, "Fetching VM SKUs, this may take a while" );
                    let skus = fetch_virtual_machine_skus(chosen_subscription.id).with_invalidation(invalidate).await?;
                    skus.into_iter().map(|sku| eyre::Ok(Choice {
                        key: format!("{} - {} - {}", sku.resource_type, sku.name, serde_json::to_string(&sku.locations)?),
                        value: sku,
                    })).collect::<eyre::Result<Vec<_>>>()
                }).await?;
                println!("{}", serde_json::to_string_pretty(&chosen_skus)?);
            }
            AzureVmBrowseOption::Publishers => AzureVmPublisherBrowseArgs.invoke().await?,
        }
        Ok(())
    }
}
