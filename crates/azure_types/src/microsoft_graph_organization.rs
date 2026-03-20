use serde::Deserialize;
use serde::Serialize;

use crate::prelude::{AzureTenantId, MicrosoftGraphDirectoryObject, MicrosoftGraphEntity};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftGraphOrganization {
    #[serde(flatten)]
    pub entity: MicrosoftGraphEntity<AzureTenantId>,
    #[serde(flatten)]
    pub directory_object: MicrosoftGraphDirectoryObject,
    pub display_name: String,
}