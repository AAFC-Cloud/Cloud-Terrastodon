use crate::ContainerRegistryId;
use crate::ContainerRegistryName;
use crate::ContainerRegistryProperties;
use crate::ContainerRegistrySKU;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use facet::Facet;

/// A container registry returned by Azure Resource Graph.
/// [Registry schema](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
pub struct ContainerRegistry {
    pub id: ContainerRegistryId,
    pub name: ContainerRegistryName,
    pub location: String,
    pub sku: ContainerRegistrySKU,
    pub properties: ContainerRegistryProperties,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContainerRegistryEndpoint;
    use std::collections::HashMap;

    #[test]
    fn endpoint_uses_the_returned_login_server_without_a_name_fallback() -> eyre::Result<()> {
        let registry = ContainerRegistry {
            id: "/subscriptions/33333333-3333-3333-3333-333333333333/resourceGroups/fixture/providers/Microsoft.ContainerRegistry/registries/fixture".parse()?,
            name: "fixture".parse()?,
            location: "fixture-region".to_owned(),
            sku: ContainerRegistrySKU {
                name: "Basic".to_owned(),
                tier: "Basic".to_owned(),
            },
            properties: ContainerRegistryProperties {
                login_server: "fixture-ab12.azurecr.io".parse()?,
                other: HashMap::new(),
            },
        };
        let endpoint = ContainerRegistryEndpoint::from(&registry);
        assert_eq!(endpoint.id, registry.id);
        assert_eq!(endpoint.login_server.as_str(), "fixture-ab12.azurecr.io");
        Ok(())
    }
}
