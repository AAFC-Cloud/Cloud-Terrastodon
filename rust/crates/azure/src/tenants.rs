use cloud_terrastodon_core_azure_types::prelude::TenantId;
use eyre::bail;

use crate::prelude::az_account_list;

pub async fn get_default_tenant_id() -> eyre::Result<TenantId> {
    let accounts = az_account_list().await?;
    let num_accounts = accounts.len();
    let Some(default_account) = accounts.into_iter().find(|account| account.is_default) else {
        bail!("Failed to find default account among {} accounts.", num_accounts);
    };
    Ok(default_account.tenant_id)
}

#[cfg(test)]
mod test {
    use crate::tenants::get_default_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant = get_default_tenant_id().await?;
        dbg!(&tenant);
        Ok(())
    }
}
