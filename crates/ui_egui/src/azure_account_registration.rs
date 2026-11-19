use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialFile;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraApplicationClientId;
use eyre::Result;
use teamy_cancellation::CancellationScope;

/// An account registration whose tenant alias can be resolved in a background job.
pub(crate) enum AzureAccountRegistration {
    AzureCli(AzureTenantArgument<'static>),
    Browser {
        tenant: AzureTenantArgument<'static>,
        client_id: EntraApplicationClientId,
    },
    WorkloadIdentity {
        tenant: AzureTenantArgument<'static>,
        client_id: EntraApplicationClientId,
        assertion_file: CloudTerrastodonCredentialFile,
    },
    Service(CloudTerrastodonAccountCredential),
}

impl AzureAccountRegistration {
    pub async fn resolve(
        self,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountCredential> {
        Ok(match self {
            Self::AzureCli(tenant) => CloudTerrastodonAccountCredential::AzureCli {
                tenant_id: tenant.resolve(cancellation).await?,
            },
            Self::Browser { tenant, client_id } => CloudTerrastodonAccountCredential::Browser {
                tenant_id: tenant.resolve(cancellation).await?,
                client_id,
            },
            Self::WorkloadIdentity {
                tenant,
                client_id,
                assertion_file,
            } => CloudTerrastodonAccountCredential::WorkloadIdentity {
                tenant_id: tenant.resolve(cancellation).await?,
                client_id,
                assertion_file,
            },
            Self::Service(credential) => credential,
        })
    }
}
