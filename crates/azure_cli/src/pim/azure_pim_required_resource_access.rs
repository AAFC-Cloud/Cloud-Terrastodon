use super::azure_pim_required_resource_access_entry::AzurePimRequiredResourceAccessEntry;
use cloud_terrastodon_azure::EntraApplicationClientId;
use facet::Facet;

/// Application resource-access declaration inspected by PIM setup.
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePimRequiredResourceAccess {
    pub resource_app_id: EntraApplicationClientId,
    pub resource_access: Vec<AzurePimRequiredResourceAccessEntry>,
}
