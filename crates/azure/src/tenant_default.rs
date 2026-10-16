use crate::az_account_list;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use eyre::bail;

pub async fn get_default_tenant_id(
    cancellation: &CancellationScope,
) -> eyre::Result<AzureTenantId> {
    let accounts = az_account_list(cancellation).await?;
    let num_accounts = accounts.len();
    let Some(default_account) = accounts.into_iter().find(|account| account.is_default) else {
        bail!(
            "Failed to find default account among {} accounts.",
            num_accounts
        );
    };
    Ok(default_account.tenant_id)
}

pub async fn get_test_tenant_id(cancellation: &CancellationScope) -> eyre::Result<AzureTenantId> {
    use crate::AzureTenantAliasExt;
    use cloud_terrastodon_azure_types::AzureTenantAlias;

    AzureTenantAlias::try_new("test")?
        .resolve(cancellation)
        .await
}

#[cfg(test)]
mod test {
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let tenant = get_test_tenant_id(&cancellation).await?;
                assert!(!tenant.to_string().is_empty());
                Ok(())
            })
            .await
    }
}
