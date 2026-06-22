use crate::AzureTenantId;
use crate::MicrosoftGraphDirectoryObject;
use crate::MicrosoftGraphEntity;

#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct MicrosoftGraphOrganization {
    #[facet(flatten)]
    pub entity: MicrosoftGraphEntity<AzureTenantId>,
    #[facet(flatten)]
    pub directory_object: MicrosoftGraphDirectoryObject,
    pub display_name: String,
}
