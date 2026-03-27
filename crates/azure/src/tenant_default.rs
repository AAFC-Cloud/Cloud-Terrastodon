use crate::prelude::az_account_list;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use eyre::bail;

pub async fn get_default_tenant_id() -> eyre::Result<AzureTenantId> {
    let accounts = az_account_list().await?;
    let num_accounts = accounts.len();
    let Some(default_account) = accounts.into_iter().find(|account| account.is_default) else {
        bail!(
            "Failed to find default account among {} accounts.",
            num_accounts
        );
    };
    Ok(default_account.tenant_id)
}

pub async fn get_test_tenant_id() -> eyre::Result<AzureTenantId> {
    use crate::prelude::AzureTenantAliasExt;
    use cloud_terrastodon_azure_types::prelude::AzureTenantAlias;

    AzureTenantAlias::try_new("test")?.resolve().await
}

#[cfg(test)]
mod test {
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant = get_test_tenant_id().await?;
        assert!(!tenant.to_string().is_empty());
        Ok(())
    }
}
