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
use facet::Facet;

#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
pub struct ContainerRegistrySKU {
    name: String,
    tier: String,
}

#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContainerRegistryEndpoint;
    use std::collections::HashMap;

    #[test]
    fn endpoint_uses_the_returned_login_server_without_a_name_fallback() -> eyre::Result<()> {
        let mut registry = ContainerRegistry {
            id: "/subscriptions/33333333-3333-3333-3333-333333333333/resourceGroups/fixture/providers/Microsoft.ContainerRegistry/registries/fixture".parse()?,
            name: "fixture".parse()?,
            location: "fixture-region".to_owned(),
            sku: ContainerRegistrySKU {
                name: "Basic".to_owned(),
                tier: "Basic".to_owned(),
            },
            properties: ArbitraryJson::try_from_facet(&HashMap::from([
                ("loginServer", ArbitraryJson::from_text("fixture-ab12.azurecr.io")),
                ("unmodeledProperty", ArbitraryJson::object()),
            ]))?,
        };
        let endpoint = ContainerRegistryEndpoint::try_from(&registry)?;
        assert_eq!(endpoint.id, registry.id);
        assert_eq!(endpoint.login_server.as_str(), "fixture-ab12.azurecr.io");

        registry.properties = ArbitraryJson::object();
        assert!(ContainerRegistryEndpoint::try_from(&registry).is_err());
        for invalid in [
            ArbitraryJson::from_text("fixture.azurecr.io/path"),
            ArbitraryJson::try_from_facet(&17_u8)?,
        ] {
            registry.properties =
                ArbitraryJson::try_from_facet(&HashMap::from([("loginServer", invalid)]))?;
            assert!(ContainerRegistryEndpoint::try_from(&registry).is_err());
        }
        Ok(())
    }
}
