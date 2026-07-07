use crate::ArbitraryJson;
use crate::ContainerRegistryId;
use crate::ContainerRegistryName;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;

#[derive(Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
pub struct ContainerRegistrySKU {
    name: String,
    tier: String,
}

#[derive(Debug, PartialEq, Eq, Arbitrary, facet::Facet)]
pub struct ContainerRegistry {
    pub id: ContainerRegistryId,
    pub name: ContainerRegistryName,
    pub location: String,
    pub sku: ContainerRegistrySKU,
    pub properties: ArbitraryJson,
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
impl From<ContainerRegistry> for HclImportBlock {
    fn from(container_registry: ContainerRegistry) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: container_registry.id.expanded_form().to_owned(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::ContainerRegistry,
                name: container_registry.name.sanitize(),
            },
        }
    }
}

cloud_terrastodon_registry::register_thing!(ContainerRegistry);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistry);
cloud_terrastodon_registry::register_arbitrary!(Vec<ContainerRegistry>);
