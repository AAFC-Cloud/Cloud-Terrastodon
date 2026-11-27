use crate::CloudTerrastodonAccountService;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
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
    BmcHelix {
        /// Scoped defaults and legacy alias namespaces identify a server.
        /// A new account registration uses the shared Helix alias namespace.
        server: Option<BmcHelixServerUrl>,
    },
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountNamespace);
