use crate::vm::browse::AzureVmBrowseOption;
use crate::vm::publisher::AzureVmPublisherBrowseArgs;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_azure::fetch_all_virtual_machines;
use cloud_terrastodon_azure::fetch_virtual_machine_skus;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<()> {
        let cancellation = executor.cancellation();
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        let auth_source = tenant_auth_context.auth_context.azure_bearer_source()?;
        let chosen = PickerTui::<_>::new()
            .pick_one(AzureVmBrowseOption::VARIANTS)
            .await?;
        match chosen {
            AzureVmBrowseOption::Resources => {
                let (chosen_resources, maybe_error) = PickerTui::<_>::new()
                    .pick_many_reloadable(|invalidate| {
                        let tenant_auth_context = tenant_auth_context.clone();
                        async move {
                            info!("Fetching virtual machines");
                            Ok(fetch_all_virtual_machines(&tenant_auth_context, executor)
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
                        }
                    })
                    .await
                    .into_chosen_and_maybe_error()?;
                println!(
                    "{}",
                    facet_json::to_string_pretty(&chosen_resources)
                        .map_err(|error| eyre::eyre!("{error:?}"))?
                );
                maybe_error?;
            }
            AzureVmBrowseOption::Skus => {
                let chosen_subscription = PickerTui::<_>::new()
                    .pick_one_reloadable(|invalidate| {
                        let tenant_auth_context = tenant_auth_context.clone();
                        async move {
                            info!("Fetching subscriptions");
                            let subscriptions =
                                fetch_all_subscriptions(&tenant_auth_context, executor)
                                    .with_invalidation(invalidate)
                                    .await?;
                            Ok(subscriptions)
                        }
                    })
                    .await?;

                let (chosen_skus, maybe_error) = PickerTui::<_>::new().pick_many_reloadable(|invalidate| {
                    let chosen_subscription = &chosen_subscription;
                    async move {
                        info!(%chosen_subscription.name, %chosen_subscription.id, "Fetching VM SKUs, this may take a while" );
                        let skus = fetch_virtual_machine_skus(chosen_subscription.id, auth_source, tenant_auth_context.tenant_id, executor).with_invalidation(invalidate).await?;
                        skus.into_iter().map(|sku| eyre::Ok(Choice {
                            key: format!("{} - {} - {}", sku.resource_type, sku.name, facet_json::to_string(&sku.locations).map_err(|error| eyre::eyre!("{error:?}"))?),
                            value: sku,
                        })).collect::<eyre::Result<Vec<_>>>()
                    }
                }).await.into_chosen_and_maybe_error()?;
                println!(
                    "{}",
                    facet_json::to_string_pretty(&chosen_skus)
                        .map_err(|error| eyre::eyre!("{error:?}"))?
                );
                maybe_error?;
            }
            AzureVmBrowseOption::Publishers => {
                AzureVmPublisherBrowseArgs {
                    tenant: self.tenant,
                }
                .invoke(&tenant_auth_context.auth_context, executor)
                .await?
            }
        }
        Ok(())
    }
}
