use crate::CloudTerrastodonAccountService;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureTenantId;
use facet::Facet;

/// Scope in which Cloud Terrastodon account aliases and defaults are resolved.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Facet)]
#[repr(u8)]
pub enum CloudTerrastodonAccountNamespace {
    Entra {
        tenant_id: AzureTenantId,
    },
    AzureDevOps {
        organization: AzureDevOpsOrganizationUrl,
    },
    Gitea {
        instance: CloudTerrastodonAccountService,
    },
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountNamespace);
