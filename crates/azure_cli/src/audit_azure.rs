use crate::AzureAdminCorrespondenceAuditRequest;
use crate::AzureAuditReport;
use crate::AzureResourceTagAuditRequest;
use crate::AzureServicePrincipalPasswordAuditRequest;
use crate::AzureUnknownPrincipalAuditRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;

/// Run the existing independent Azure policies through the shared inventory cache.
pub async fn audit_azure(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<AzureAuditReport> {
    let observed_at = chrono::Utc::now();
    let (mut unknown, passwords, tags, admins) = executor
        .join4(
            async |child| {
                AzureUnknownPrincipalAuditRequest { authentication }
                    .execute(&child)
                    .await
            },
            async |child| {
                AzureServicePrincipalPasswordAuditRequest {
                    authentication,
                    observed_at,
                }
                .execute(&child)
                .await
            },
            async |child| {
                AzureResourceTagAuditRequest { authentication }
                    .execute(&child)
                    .await
            },
            async |child| {
                AzureAdminCorrespondenceAuditRequest { authentication }
                    .execute(&child)
                    .await
            },
        )
        .await?;
    unknown.extend(passwords);
    unknown.extend(tags);
    unknown.extend(admins);
    Ok(AzureAuditReport { findings: unknown })
}
