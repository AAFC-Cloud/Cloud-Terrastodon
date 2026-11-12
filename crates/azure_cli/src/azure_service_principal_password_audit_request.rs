use crate::AzureAuditFinding;
use crate::AzureAuditPasswordCredentialStatus;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraPrincipalCollection;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;

/// Check service-principal password expiry against the existing 30-day policy.
#[derive(Debug, Arbitrary, Facet)]
pub struct AzureServicePrincipalPasswordAuditRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub observed_at: DateTime<Utc>,
}

impl AzureServicePrincipalPasswordAuditRequest {
    pub fn evaluate(
        principals: &EntraPrincipalCollection,
        now: DateTime<Utc>,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Vec<AzureAuditFinding>> {
        let mut findings = Vec::new();
        for principal in principals
            .values()
            .filter_map(|principal| principal.as_service_principal())
        {
            cancellation.bail_if_cancelled()?;
            let replacement = principal
                .password_credentials
                .iter()
                .any(|password| (password.end_date_time - now).num_days() > 30);
            for password in &principal.password_credentials {
                let days_until_expiry = (password.end_date_time - now).num_days();
                let status = if days_until_expiry < 0 && !replacement {
                    AzureAuditPasswordCredentialStatus::ExpiredWithoutReplacement
                } else if days_until_expiry < 30 && !replacement {
                    AzureAuditPasswordCredentialStatus::ExpiringWithoutReplacement
                } else if days_until_expiry < 0 && replacement {
                    AzureAuditPasswordCredentialStatus::ExpiredWithReplacement
                } else {
                    continue;
                };
                findings.push(AzureAuditFinding::ServicePrincipalPassword {
                    principal_id: principal.id,
                    principal_name: principal.display_name.clone(),
                    password_cred_end_date: password.end_date_time,
                    days_until_expiry,
                    status,
                });
            }
        }
        Ok(findings)
    }
}

impl ExecutableRequest for AzureServicePrincipalPasswordAuditRequest {
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
            Self::evaluate(&principals, self.observed_at, executor.cancellation())
        })
    }
}

cloud_terrastodon_registry_execution::register_executable_request!(AzureServicePrincipalPasswordAuditRequest => Vec<AzureAuditFinding>, effects = [Read]);

cloud_terrastodon_registry::register_thing!(AzureServicePrincipalPasswordAuditRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureServicePrincipalPasswordAuditRequest);
