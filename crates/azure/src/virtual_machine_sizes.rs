use crate::VirtualMachineSizeListRequest;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureLocationName;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;

#[track_caller]
pub fn fetch_virtual_machine_sizes<'a>(
    subscription_id: &SubscriptionId,
    location: &AzureLocationName,
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, VirtualMachineSizeListRequest<'a>> {
    executor.execute(VirtualMachineSizeListRequest {
        subscription_id: *subscription_id,
        location: location.clone(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

#[cfg(test)]
mod test {
    use crate::fetch_all_subscriptions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::AzureLocationName;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let subs = fetch_all_subscriptions(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?;
                let sub = subs.first().unwrap();
                let sizes = crate::fetch_virtual_machine_sizes(
                    &sub.id,
                    &AzureLocationName::CanadaCentral,
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?;
                assert!(!sizes.is_empty());
                Ok(())
            })
            .await
    }
}
