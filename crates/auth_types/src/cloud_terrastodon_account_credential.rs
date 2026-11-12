use crate::AuthSource;
use crate::CloudTerrastodonAccountNamespace;
use crate::CloudTerrastodonAccountService;
use crate::CloudTerrastodonCredentialFile;
use crate::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use facet::Facet;

/// Non-secret acquisition configuration of one account's credential mechanism.
#[derive(Debug, Clone, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum CloudTerrastodonAccountCredential {
    AzureCli {
        tenant_id: AzureTenantId,
    },
    Browser {
        tenant_id: AzureTenantId,
        client_id: EntraApplicationClientId,
    },
    WorkloadIdentity {
        tenant_id: AzureTenantId,
        client_id: EntraApplicationClientId,
        assertion_file: CloudTerrastodonCredentialFile,
    },
    AzureDevOpsPersonalAccessToken {
        organization: AzureDevOpsOrganizationUrl,
    },
    Gitea {
        instance: CloudTerrastodonAccountService,
        login: String,
        configuration: CloudTerrastodonCredentialFile,
    },
}
impl CloudTerrastodonAccountCredential {
    pub const fn source(&self) -> CloudTerrastodonCredentialSource {
        match self {
            Self::AzureCli { .. } => CloudTerrastodonCredentialSource::AzureCli,
            Self::Browser { .. } => CloudTerrastodonCredentialSource::Browser,
            Self::WorkloadIdentity { .. } => CloudTerrastodonCredentialSource::WorkloadIdentity,
            Self::AzureDevOpsPersonalAccessToken { .. } => {
                CloudTerrastodonCredentialSource::PersonalAccessToken
            }
            Self::Gitea { .. } => CloudTerrastodonCredentialSource::Gitea,
        }
    }
    pub fn namespace(&self) -> CloudTerrastodonAccountNamespace {
        match self {
            Self::AzureCli { tenant_id }
            | Self::Browser { tenant_id, .. }
            | Self::WorkloadIdentity { tenant_id, .. } => CloudTerrastodonAccountNamespace::Entra {
                tenant_id: *tenant_id,
            },
            Self::AzureDevOpsPersonalAccessToken { organization } => {
                CloudTerrastodonAccountNamespace::AzureDevOps {
                    organization: organization.clone(),
                }
            }
            Self::Gitea { instance, .. } => CloudTerrastodonAccountNamespace::Gitea {
                instance: instance.clone(),
            },
        }
    }
    pub const fn auth_source(&self) -> Option<AuthSource> {
        match self {
            Self::AzureCli { .. } => Some(AuthSource::AzureCli),
            Self::Browser { .. } => Some(AuthSource::Browser),
            Self::WorkloadIdentity { .. } => Some(AuthSource::WorkloadIdentity),
            Self::AzureDevOpsPersonalAccessToken { .. } => Some(AuthSource::PersonalAccessToken),
            Self::Gitea { .. } => None,
        }
    }
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountCredential);
