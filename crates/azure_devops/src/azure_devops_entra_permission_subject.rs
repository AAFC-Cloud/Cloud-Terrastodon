use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraUserId;

/// The Entra object to correlate with Azure DevOps Graph identities.
#[derive(Debug, Clone, Copy)]
pub enum AzureDevOpsEntraPermissionSubject {
    User(EntraUserId),
    Group(EntraGroupId),
}
