use crate::AzureTenantId;
use crate::MicrosoftGraphDirectoryObject;
use crate::MicrosoftGraphEntity;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct MicrosoftGraphOrganization {
    #[facet(flatten)]
    pub entity: MicrosoftGraphEntity<AzureTenantId>,
    #[facet(flatten)]
    pub directory_object: MicrosoftGraphDirectoryObject,
    pub display_name: String,
}

cloud_terrastodon_registry::register_thing!(MicrosoftGraphOrganization);
cloud_terrastodon_registry::register_arbitrary!(MicrosoftGraphOrganization);
