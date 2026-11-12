use crate::AuthContext;
use crate::CredentialAuthenticationProvider;
use crate::ResolvedAuthContext;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::AuthenticationProvider;
use eyre::Result;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn one_accounts_mechanisms_have_distinct_partitions_and_independent_retirement() -> Result<()>
{
    let account_id = CloudTerrastodonAccountId::new(Uuid::from_u128(1));
    let tenant_id = AzureTenantId::new(Uuid::from_u128(2));
    let contexts = [AuthSource::AzureCli, AuthSource::Browser].map(|source| {
        let mut context = ResolvedAuthContext::new(source, None, false, None);
        context.account_id = account_id;
        context.tenant_id = Some(tenant_id);
        AuthContext::Resolved(Arc::new(context))
    });
    let provider = CredentialAuthenticationProvider::from_accounts(contexts)?;
    let cli = AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
        account_id,
        tenant_id,
        auth_source: AzureBearerAuthSource::AzureCli,
    });
    let browser = AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
        account_id,
        tenant_id,
        auth_source: AzureBearerAuthSource::Browser,
    });
    assert_ne!(provider.admit(&cli)?, provider.admit(&browser)?);
    assert_eq!(provider.admit(&cli)?, provider.clone().admit(&cli)?);
    let retained_browser = provider.lease(&browser)?.expect("retained browser lease");
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let cache = WorkCache::temporary()?;
            // This must finish despite the unrelated mechanism's retained lease.
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                provider.retire_source(
                    account_id,
                    CloudTerrastodonCredentialSource::AzureCli,
                    &cache,
                    &scope,
                ),
            )
            .await??;
            assert!(provider.admit(&cli).is_err());
            assert!(provider.lease(&cli).is_err());
            assert!(provider.admit(&browser).is_ok());
            assert!(retained_browser.admit(&browser).is_ok());
            assert!(
                retained_browser
                    .retirement_tokens()
                    .iter()
                    .all(|token| !token.is_cancelled())
            );
            drop(retained_browser);
            provider.retire(account_id, &cache, &scope).await?;
            assert!(provider.admit(&cli).is_err());
            assert!(provider.admit(&browser).is_err());
            eyre::Ok(())
        })
        .await
}
