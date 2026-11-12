use crate::AzureAuditFinding;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraPrincipalCollection;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::collections::HashSet;

/// Check admin-prefixed users for a corresponding inventory UPN in other_mails.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureAdminCorrespondenceAuditRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl CacheInvalidationKeys for AzureAdminCorrespondenceAuditRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        EntraPrincipalListRequest {
            authentication: self.authentication,
        }
        .cache_keys_to_invalidate()
    }
}

impl AzureAdminCorrespondenceAuditRequest {
    pub fn evaluate(
        principals: &EntraPrincipalCollection,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Vec<AzureAuditFinding>> {
        let users = principals
            .values()
            .filter_map(|principal| principal.as_user());
        let names = users
            .clone()
            .map(|user| user.user_principal_name.to_lowercase())
            .collect::<HashSet<_>>();
        let mut findings = Vec::new();
        for user in users.filter(|user| {
            user.user_principal_name
                .to_lowercase()
                .starts_with("admin.")
        }) {
            cancellation.bail_if_cancelled()?;
            // Preserve the existing correspondence policy: any inventoried UPN
            // is a match; this extraction does not add an admin-status filter.
            if !user
                .other_mails
                .iter()
                .any(|mail| names.contains(&mail.to_lowercase()))
            {
                findings.push(AzureAuditFinding::AdminWithoutCorrespondingAccount {
                    principal_id: user.id,
                    principal_name: user.display_name.clone(),
                    principal_other_mails: user.other_mails.clone(),
                });
            }
        }
        Ok(findings)
    }
}

impl ExecutableRequest for AzureAdminCorrespondenceAuditRequest {
    type Output = Vec<AzureAuditFinding>;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let principals = EntraPrincipalListRequest {
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            Self::evaluate(&principals, executor.cancellation())
        })
    }
}

cloud_terrastodon_registry_execution::register_executable_request!(AzureAdminCorrespondenceAuditRequest => Vec<AzureAuditFinding>, effects = [Read]);

cloud_terrastodon_registry::register_thing!(AzureAdminCorrespondenceAuditRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureAdminCorrespondenceAuditRequest);
