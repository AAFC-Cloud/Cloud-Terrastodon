use crate::prelude::ContainerRegistryId;
use crate::prelude::ContainerRegistryName;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerRegistrySKU {
    name: String,
    tier: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerRegistry {
    pub id: ContainerRegistryId,
    pub name: ContainerRegistryName,
    pub location: String,
    pub sku: ContainerRegistrySKU,
    pub properties: Value,
}

impl AsScope for ContainerRegistry {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &ContainerRegistry {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for ContainerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}
impl From<ContainerRegistry> for HCLImportBlock {
    fn from(container_registry: ContainerRegistry) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: container_registry.id.expanded_form().to_owned(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::ContainerRegistry,
                name: container_registry.name.sanitize(),
            },
        }
    }
}