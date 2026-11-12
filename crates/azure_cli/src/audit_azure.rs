use crate::AzureAdminCorrespondenceAuditRequest;
use crate::AzureAuditReport;
use crate::AzureResourceTagAuditRequest;
use crate::AzureServicePrincipalPasswordAuditRequest;
use crate::AzureUnknownPrincipalAuditRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;

/// Run the existing independent Azure policies through the shared inventory cache.
pub async fn audit_azure(
    authentication: AzureBearerAuthenticationSelection,
    clean: bool,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<AzureAuditReport> {
    let unknown = AzureUnknownPrincipalAuditRequest { authentication };
    let passwords = AzureServicePrincipalPasswordAuditRequest {
        authentication,
        observed_at: chrono::Utc::now(),
    };
    let tags = AzureResourceTagAuditRequest { authentication };
    let admins = AzureAdminCorrespondenceAuditRequest { authentication };
    if clean {
        // Finish all invalidations before starting the shared inventory reads.
        // Refreshing inside each concurrent audit could retire another audit's
        // in-flight principal fill and cause redundant requests.
        unknown.invalidate(executor).await?;
        passwords.invalidate(executor).await?;
        tags.invalidate(executor).await?;
        admins.invalidate(executor).await?;
    }
    let (mut unknown, passwords, tags, admins) = executor
        .join4(
            async |child| unknown.execute(&child).await,
            async |child| passwords.execute(&child).await,
            async |child| tags.execute(&child).await,
            async |child| admins.execute(&child).await,
        )
        .await?;
    unknown.extend(passwords);
    unknown.extend(tags);
    unknown.extend(admins);
    Ok(AzureAuditReport { findings: unknown })
}
