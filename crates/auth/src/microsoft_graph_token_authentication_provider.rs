use crate::MicrosoftGraphAccessToken;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use eyre::ensure;
use uuid::Uuid;

/// An invocation-owned Graph credential acquired before executing SDK requests.
///
/// This supports the delegated [PIM workflow](https://learn.microsoft.com/graph/api/resources/privilegedidentitymanagementv3-overview?view=graph-rest-1.0)
/// without storing a token in a request. The credential never refreshes or selects
/// another account; expiry requires a new provider and partition.
#[derive(Clone)]
pub struct MicrosoftGraphTokenAuthenticationProvider {
    // Selection, authorization and partition are immutable together: replacing
    // any one could reuse results under a different credential identity.
    selection: AuthenticationSelection,
    authorization: Authorization,
    partition: AuthPartition,
}

impl MicrosoftGraphTokenAuthenticationProvider {
    pub fn new(
        auth_source: AzureBearerAuthSource,
        tenant_id: AzureTenantId,
        token: MicrosoftGraphAccessToken,
    ) -> Result<Self> {
        Ok(Self {
            selection: AuthenticationSelection::AzureBearer {
                auth_source,
                tenant_id,
            },
            authorization: Authorization::bearer(token.as_str())?,
            partition: AuthPartition::new(format!("session/{}/graph-token", Uuid::new_v4()))?,
        })
    }

    /// The exact source and tenant admitted for the captured credential.
    pub fn selection(&self) -> AuthenticationSelection {
        self.selection
    }
}

impl AuthenticationProvider for MicrosoftGraphTokenAuthenticationProvider {
    fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        if *selection == AuthenticationSelection::Anonymous {
            return AuthPartition::new("anonymous");
        }
        ensure!(
            *selection == self.selection,
            "the captured Microsoft Graph credential is bound to another source or tenant"
        );
        Ok(self.partition.clone())
    }

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
        _executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            self.admit(selection)?;
            if *selection == AuthenticationSelection::Anonymous {
                return Ok(Authorization::Anonymous);
            }
            ensure!(
                resource == AzureCredentialResource::MicrosoftGraph,
                "the captured Microsoft Graph credential cannot authorize another audience"
            );
            Ok(self.authorization.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use std::sync::Arc;

    #[tokio::test]
    async fn captured_credentials_preserve_partition_and_reject_another_audience() -> Result<()> {
        let tenant = "11111111-1111-1111-1111-111111111111".parse()?;
        let make = || {
            MicrosoftGraphTokenAuthenticationProvider::new(
                AzureBearerAuthSource::Browser,
                tenant,
                MicrosoftGraphAccessToken::new("synthetic-graph-credential".to_owned()),
            )
        };
        let provider = make()?;
        let selection = provider.selection();
        assert_eq!(
            provider.admit(&selection)?,
            provider.clone().admit(&selection)?
        );
        assert_ne!(provider.admit(&selection)?, make()?.admit(&selection)?);
        assert!(
            provider
                .admit(&AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::AzureCli,
                    tenant_id: tenant,
                })
                .is_err()
        );
        assert!(
            provider
                .admit(&AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::Browser,
                    tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
                })
                .is_err()
        );
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(provider),
                    scope,
                );
                assert!(
                    executor
                        .authorization(&selection, AzureCredentialResource::AzureResourceManager)
                        .await
                        .is_err()
                );
                let authorization = executor
                    .authorization(&selection, AzureCredentialResource::MicrosoftGraph)
                    .await?;
                assert!(
                    authorization
                        .header()
                        .is_some_and(|header| header.is_sensitive())
                );
                assert!(!format!("{authorization:?}").contains("synthetic-graph-credential"));
                Ok(())
            })
            .await
    }
}
