use crate::portal_tab_id::PortalTabId;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;

/// Retains the initiating view and the picker selection it requested.
pub(crate) struct AzureAccountOperationOrigin {
    pub tab: PortalTabId,
    pub generation: u64,
    pub picker_selection: Option<AzureBearerAuthenticationSelection>,
}
