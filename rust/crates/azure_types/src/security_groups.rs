use cloud_terrasotodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrasotodon_core_tofu_types::prelude::TofuAzureADResourceKind;
use cloud_terrasotodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrasotodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrasotodon_core_tofu_types::prelude::TofuResourceReference;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityGroup {
    pub id: Uuid,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

impl From<SecurityGroup> for TofuImportBlock {
    fn from(security_group: SecurityGroup) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: security_group.id.to_string(),
            to: TofuResourceReference::AzureAD {
                kind: TofuAzureADResourceKind::Group,
                name: format!("{}__{}", security_group.display_name, security_group.id).sanitize(),
            },
        }
    }
}
