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
}

impl AzureCredentialResource {
    /// Application scope used by the workload-identity token exchange.
    pub const fn scope(self) -> &'static str {
        match self {
            Self::MicrosoftGraph => "https://graph.microsoft.com/.default",
            Self::AzureResourceManager => "https://management.azure.com/.default",
            Self::AzureDevOps => "499b84ac-1321-427f-aa17-267ca6975798/.default",
        }
    }

    /// Concrete delegated permission requested by the existing browser flow.
    ///
    /// This deliberately preserves that flow's scope selection separately from
    /// the workload-identity default scope. It does not claim that `.default`
    /// is restricted to application authentication.
    pub const fn delegated_scope(self) -> &'static str {
        match self {
            Self::MicrosoftGraph => "https://graph.microsoft.com/User.Read",
            Self::AzureResourceManager => "https://management.azure.com/user_impersonation",
            Self::AzureDevOps => AZURE_DEVOPS_DELEGATED_SCOPE,
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureCredentialResource);
cloud_terrastodon_registry::register_arbitrary!(AzureCredentialResource);
