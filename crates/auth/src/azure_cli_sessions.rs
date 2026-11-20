use crate::AzureBearerToken;
use crate::AzureCliAccountName;
use crate::azure_cli_session::AzureCliSession;
use crate::azure_cli_session_store::AzureCliSessionStore;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Invocation-owned bindings to persistent, tenant-specific Azure CLI logins.
/// Clones share the same bindings; later explicit logins cannot replace them.
#[derive(Clone, Default)]
pub(crate) struct AzureCliSessions {
    sessions: Arc<Mutex<HashMap<AzureTenantId, Arc<AzureCliSession>>>>,
    store: Option<AzureCliSessionStore>,
}

impl AzureCliSessions {
    pub(crate) fn at(root: std::path::PathBuf) -> Self {
        Self {
            sessions: Arc::default(),
            store: Some(AzureCliSessionStore::at(root)),
        }
    }
    async fn session(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<Arc<AzureCliSession>> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get(&tenant) {
            return Ok(session.clone());
        }
        let session = Arc::new(
            self.store
                .as_ref()
                .ok_or_else(|| eyre::eyre!("no named-account Azure CLI store was supplied"))?
                .load(tenant, cancellation)
                .await?,
        );
        sessions.insert(tenant, session.clone());
        Ok(session)
    }

    pub(crate) async fn fetch_bearer_access_token(
        &self,
        tenant: AzureTenantId,
        resource: AzureCredentialResource,
        cancellation: &CancellationScope,
    ) -> Result<AzureAccessToken<AzureBearerToken>> {
        self.session(tenant, cancellation)
            .await?
            .fetch_token(resource, cancellation)
            .await
    }

    pub(crate) async fn sign_in_name(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<AzureCliAccountName> {
        let session = self.session(tenant, cancellation).await?;
        let _lease = session.lease(cancellation).await?;
        Ok(session.identity.account_name.clone())
    }
}

impl std::fmt::Debug for AzureCliSessions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("AzureCliSessions([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::azure_cli_account_identity::AzureCliAccountIdentity;
    use cloud_terrastodon_azure_types::AzureCliAccountUserType;
    use cloud_terrastodon_command::CancellationToken;
    use uuid::Uuid;

    #[tokio::test]
    async fn sign_in_name_stays_bound_and_disappears_after_revocation() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        let tenant = AzureTenantId::new(Uuid::from_u128(1));
        let first = Uuid::from_u128(2);
        let second = Uuid::from_u128(3);
        for (generation, name) in [
            (first, "first@example.invalid"),
            (second, "second@example.invalid"),
        ] {
            let path = root.join("generations").join(generation.to_string());
            std::fs::create_dir_all(&path)?;
            std::fs::write(
                path.join("identity.json"),
                facet_json::to_vec(&AzureCliAccountIdentity {
                    tenant,
                    account_name: name.to_owned().try_into()?,
                    account_type: AzureCliAccountUserType::User,
                })?,
            )?;
        }
        std::fs::create_dir(root.join("tenants"))?;
        let selected = root.join("tenants").join(tenant.to_string());
        std::fs::write(&selected, first.to_string())?;
        let sessions = AzureCliSessions::at(root.to_path_buf());
        let expected_name = AzureCliAccountName::try_from("first@example.invalid".to_owned())?;
        CancellationToken::new()
            .with_async_scope(async |scope| {
                assert_eq!(sessions.sign_in_name(tenant, &scope).await?, expected_name);
                std::fs::write(&selected, second.to_string())?;
                assert_eq!(sessions.sign_in_name(tenant, &scope).await?, expected_name);
                std::fs::write(
                    root.join("generations")
                        .join(first.to_string())
                        .join("revoked"),
                    b"logged out",
                )?;
                assert!(sessions.sign_in_name(tenant, &scope).await.is_err());
                eyre::Ok(())
            })
            .await
    }
}
