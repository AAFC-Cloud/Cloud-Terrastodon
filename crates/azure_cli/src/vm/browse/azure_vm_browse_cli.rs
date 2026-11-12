use crate::vm::browse::AzureVmBrowseOption;
use crate::vm::publisher::AzureVmPublisherBrowseArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::AzureVirtualMachineListRequest;
use cloud_terrastodon_azure::AzureVirtualMachineSkuRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use facet::Facet;
use strum::VariantArray;
use tracing::info;

#[derive(Facet, Debug, Clone)]
pub struct AzureVmBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureVmBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> eyre::Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let chosen = PickerTui::<_>::new()
            .pick_one(AzureVmBrowseOption::VARIANTS, executor.cancellation())
            .await?;
        match chosen {
            AzureVmBrowseOption::Resources => {
                let (chosen_resources, maybe_error) = PickerTui::<_>::new()
                    .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
                        let executor = &executor.with_cancellation_scope(child);
                        info!("Fetching virtual machines");
                        Ok(AzureVirtualMachineListRequest { authentication }
                            .execute(executor)
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
                    .await
                    .into_chosen_and_maybe_error()?;
                Ok(CliOutput::facet(chosen_resources).with_result(maybe_error))
            }
            AzureVmBrowseOption::Skus => {
                let chosen_subscription = PickerTui::<_>::new()
                    .pick_one_reloadable(executor.cancellation(), async |child, invalidate| {
                        let executor = &executor.with_cancellation_scope(child);
                        info!("Fetching subscriptions");
                        let subscriptions = AzureSubscriptionListRequest { authentication }
                            .execute(executor)
                            .with_invalidation(invalidate)
                            .await?;
                        Ok(subscriptions)
                    })
                    .await?;

                let (chosen_skus, maybe_error) = PickerTui::<_>::new().pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
            let executor = &executor.with_cancellation_scope(child);
                        info!(%chosen_subscription.name, %chosen_subscription.id, "Fetching VM SKUs, this may take a while" );
                        let skus = AzureVirtualMachineSkuRequest { authentication, subscription_id: (chosen_subscription.id) }.execute(executor).with_invalidation(invalidate).await?;
                        skus.into_iter().map(|sku| eyre::Ok(Choice {
                            key: format!("{} - {} - {}", sku.resource_type, sku.name, facet_json::to_string(&sku.locations).map_err(|error| eyre::eyre!("{error:?}"))?),
                            value: sku,
                        })).collect::<eyre::Result<Vec<_>>>()
                }).await.into_chosen_and_maybe_error()?;
                Ok(CliOutput::facet(chosen_skus).with_result(maybe_error))
            }
            AzureVmBrowseOption::Publishers => {
                AzureVmPublisherBrowseArgs {
                    authentication: self.authentication,
                }
                .invoke(executor)
                .await
            }
        }
    }
}
