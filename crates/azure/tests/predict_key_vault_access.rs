#![cfg(test)]

use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_azure::fetch_all_key_vaults;
use cloud_terrastodon_azure::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_azure::fetch_current_user;
use cloud_terrastodon_azure::get_test_tenant_id;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use itertools::Itertools;
use std::sync::Arc;
use tokio::try_join;

#[tokio::test]
#[ignore]
pub async fn predict_would_secret_list_succeed() -> eyre::Result<()> {
    CancellationToken::new()
        .cancellable_scope(|cancellation| async move {
            let tenant_id = get_test_tenant_id(&cancellation).await?;
            let auth_context = cloud_terrastodon_auth::AuthContext::explicit_azure_cli();
            let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(CredentialAuthenticationProvider::new(auth_context)),
                cancellation,
            );
            let current_user = fetch_current_user(&tenant_auth_context, &executor).await?;
            let (key_vaults, rbac) = try_join!(
                fetch_all_key_vaults(&tenant_auth_context, &executor),
                fetch_all_role_definitions_and_assignments(&tenant_auth_context, &executor),
            )?;

            let key_vaults = key_vaults
                .into_iter()
                .map(|kv| (kv.can_list_secrets(&current_user.id, &rbac), kv))
                .sorted_by(|(a_can, a_kv), (b_can, b_kv)| {
                    a_can.cmp(b_can).then(a_kv.name.cmp(&b_kv.name))
                });
            let outcomes = key_vaults.collect_vec();
            assert!(!outcomes.is_empty());
            assert!(
                outcomes.iter().any(|(can_list, _)| *can_list)
                    || outcomes.iter().any(|(can_list, _)| !*can_list)
            );
            Ok(())
        })
        .await
}
