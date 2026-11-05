use arbitrary::Arbitrary;
use facet::Facet;

/// Azure DevOps' Microsoft Entra resource application identifier.
/// See [Azure DevOps resource identity](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra-oauth?view=azure-devops#tips-for-building-and-migrating).
pub const AZURE_DEVOPS_RESOURCE_ID: &str = "499b84ac-1321-427f-aa17-267ca6975798";

/// The delegated scope used by the existing Azure DevOps browser flow.
/// See [Microsoft's Azure DevOps delegated-scope guidance](https://devblogs.microsoft.com/devops/new-azure-devops-scopes-now-available-for-microsoft-identity-oauth-delegated-flow-apps/).
pub const AZURE_DEVOPS_DELEGATED_SCOPE: &str =
    "499b84ac-1321-427f-aa17-267ca6975798/user_impersonation";

/// The resource audience for an acquired Microsoft Entra credential.
///
/// This owns audience and scope vocabulary without performing token acquisition
/// or constructing commands. Values preserve the existing public-cloud scopes.
/// See [Microsoft identity scopes and permissions](https://learn.microsoft.com/en-us/entra/identity-platform/scopes-oidc#the-default-scope).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureCredentialResource {
    MicrosoftGraph,
    AzureResourceManager,
    AzureDevOps,
    /// [Key Vault data-plane audience](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#security).
    AzureKeyVault,
    /// [Container Registry data-plane audience](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-tags?view=rest-registry-dataplane-2021-07-01#security).
    AzureContainerRegistry,
}

impl AzureCredentialResource {
    /// Application scope used by the workload-identity token exchange.
    pub const fn scope(self) -> &'static str {
        match self {
            Self::MicrosoftGraph => "https://graph.microsoft.com/.default",
            Self::AzureResourceManager => "https://management.azure.com/.default",
            Self::AzureDevOps => "499b84ac-1321-427f-aa17-267ca6975798/.default",
            Self::AzureKeyVault => "https://vault.azure.net/.default",
            Self::AzureContainerRegistry => "https://containerregistry.azure.net/.default",
        }
    }

    /// Scope requested by the browser flow for this resource.
    ///
    /// Named delegated permissions are kept separate from workload identity's
    /// default scopes. ACR exposes only `.default`; that scope is also valid for
    /// delegated token acquisition and must not be mixed with named permissions.
    pub const fn delegated_scope(self) -> &'static str {
        match self {
            Self::MicrosoftGraph => "https://graph.microsoft.com/User.Read",
            Self::AzureResourceManager => "https://management.azure.com/user_impersonation",
            Self::AzureDevOps => AZURE_DEVOPS_DELEGATED_SCOPE,
            Self::AzureKeyVault => "https://vault.azure.net/user_impersonation",
            // ACR exposes no named delegated permission. Its documented scope is .default.
            Self::AzureContainerRegistry => "https://containerregistry.azure.net/.default",
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureCredentialResource);
cloud_terrastodon_registry::register_arbitrary!(AzureCredentialResource);
