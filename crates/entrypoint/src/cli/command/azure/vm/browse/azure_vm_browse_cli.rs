use crate::cli::azure::vm::browse::AzureVmBrowseOption;
use crate::cli::azure::vm::publisher::AzureVmPublisherBrowseArgs;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_azure::fetch_all_virtual_machines;
use cloud_terrastodon_azure::fetch_virtual_machine_skus;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use strum::VariantArray;
use tracing::info;

#[derive(facet::Facet, Debug, Clone)]
pub struct AzureVmBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureVmBrowseArgs {
    pub async fn invoke(self) -> eyre::Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let chosen = PickerTui::<_>::new()
            .pick_one(AzureVmBrowseOption::VARIANTS)
            .await?;
        match chosen {
            AzureVmBrowseOption::Resources => {
                let chosen_resources = PickerTui::<_>::new()
                    .pick_many_reloadable(|invalidate| async move {
                        info!("Fetching virtual machines");
                        Ok(fetch_all_virtual_machines(tenant_id)
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
                println!(
                    "{}",
                    cloud_terrastodon_command::to_string_pretty(&chosen_resources)?
                );
            }
            AzureVmBrowseOption::Skus => {
                let chosen_subscription = PickerTui::<_>::new()
                    .pick_one_reloadable(|invalidate| async move {
                        info!("Fetching subscriptions");
                        let subscriptions = fetch_all_subscriptions(tenant_id)
                            .with_invalidation(invalidate)
                            .await?;
                        Ok(subscriptions)
                    })
                    .await?;

                let chosen_skus = PickerTui::<_>::new().pick_many_reloadable(|invalidate| {
                    let chosen_subscription = &chosen_subscription;
                    async move {
                        info!(%chosen_subscription.name, %chosen_subscription.id, "Fetching VM SKUs, this may take a while" );
                        let skus = fetch_virtual_machine_skus(chosen_subscription.id).with_invalidation(invalidate).await?;
                        skus.into_iter().map(|sku| eyre::Ok(Choice {
                            key: format!("{} - {} - {}", sku.resource_type, sku.name, cloud_terrastodon_command::to_string(&sku.locations)?),
                            value: sku,
                        })).collect::<eyre::Result<Vec<_>>>()
                    }
                }).await?;
                println!(
                    "{}",
                    cloud_terrastodon_command::to_string_pretty(&chosen_skus)?
                );
            }
            AzureVmBrowseOption::Publishers => {
                AzureVmPublisherBrowseArgs {
                    tenant: self.tenant,
                }
                .invoke()
                .await?
            }
        }
        Ok(())
    }
}
