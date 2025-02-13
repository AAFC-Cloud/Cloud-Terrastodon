use eyre::bail;
use eyre::OptionExt;
use hcl::edit::expr::Object;
use hcl::edit::expr::ObjectKey;
use hcl::edit::expr::ObjectValue;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl::edit::Decorated;
use hcl_primitives::Ident;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TofuTerraformBlock {
    pub backend: Option<TofuTerraformBackendBlock>,
    pub required_providers: Option<TofuTerraformRequiredProvidersBlock>,
    pub other: Vec<Structure>,
}
impl From<TofuTerraformBlock> for Block {
    fn from(block: TofuTerraformBlock) -> Self {
        let mut builder = Block::builder(Ident::new("terraform"));
        if let Some(backend) = block.backend {
            let backend: Block = backend.into();
            builder = builder.block(backend);
        }
        if let Some(required_providers) = block.required_providers {
            let required_providers: Block = required_providers.into();
            builder = builder.block(required_providers);
        }
        builder.build()
    }
}
impl TryFrom<Block> for TofuTerraformBlock {
    type Error = eyre::Error;

    fn try_from(block: Block) -> eyre::Result<Self> {
        if block.ident.to_string() != "terraform" {
            bail!("Block must use 'terraform' ident");
        }
        if !block.labels.is_empty() {
            bail!("Block must have exactly zero labels");
        }
        let mut other: Vec<Structure> = Vec::new();
        let mut backend: Option<TofuTerraformBackendBlock> = None;
        let mut required_providers: Option<TofuTerraformRequiredProvidersBlock> = None;

        for structure in block.body.into_iter() {
            match structure.into_block() {
                Ok(block) => {
                    if block.has_ident("backend") {
                        if backend.is_some() {
                            bail!("Expected at most one backend block")
                        }
                        backend = Some(block.try_into()?);
                    } else if block.has_ident("required_providers") {
                        if required_providers.is_some() {
                            bail!("Expected at most one required_providers block");
                        }
                        required_providers = Some(block.try_into()?);
                    } else {
                        // bail!("Unexpected block with ident {:?}", block.ident.to_string());
                        other.push(Structure::Block(block));
                    }
                }
                Err(structure) => {
                    // bail!("Unexpected structure: {:?}", structure);
                    other.push(structure);
                }
            }
        }

        let this = TofuTerraformBlock {
            backend,
            required_providers,
            other,
        };
        Ok(this)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TofuTerraformBackendBlock {
    AzureRM(TofuTerraformAzureRMBackendBlock),
    Other { label: String, body: Body },
}

impl From<TofuTerraformBackendBlock> for Block {
    fn from(backend: TofuTerraformBackendBlock) -> Self {
        match backend {
            TofuTerraformBackendBlock::AzureRM(block) => block.into(),
            TofuTerraformBackendBlock::Other { label, body } => {
                let mut builder = Block::builder(Ident::new("backend")).label(label);
                builder = builder.structures(body);
                builder.build()
            }
        }
    }
}

impl TryFrom<Block> for TofuTerraformBackendBlock {
    type Error = eyre::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let "backend" = block.ident.as_str() else {
            bail!("Block must use ident 'backend'");
        };
        if block.has_exact_labels(&["azurerm"]) {
            Ok(TofuTerraformBackendBlock::AzureRM(block.try_into()?))
        } else {
            Ok(TofuTerraformBackendBlock::Other {
                label: block
                    .labels
                    .iter()
                    .collect_tuple::<(_,)>()
                    .ok_or_eyre("Missing label on backend block")?
                    .0
                    .to_string(),
                body: block.body,
            })
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TofuTerraformAzureRMBackendBlock {
    pub resource_group_name: String,
    pub storage_account_name: String,
    pub container_name: String,
    pub subscription_id: String,
    pub key: String,
}
impl From<TofuTerraformAzureRMBackendBlock> for Block {
    fn from(block: TofuTerraformAzureRMBackendBlock) -> Self {
        let mut builder = Block::builder(Ident::new("backend")).label("azurerm");
        builder = builder.attribute(Attribute::new(
            Ident::new("resource_group_name"),
            block.resource_group_name,
        ));
        builder = builder.attribute(Attribute::new(
            Ident::new("storage_account_name"),
            block.storage_account_name,
        ));
        builder = builder.attribute(Attribute::new(
            Ident::new("container_name"),
            block.container_name,
        ));
        builder = builder.attribute(Attribute::new(
            Ident::new("subscription_id"),
            block.subscription_id,
        ));
        builder = builder.attribute(Attribute::new(Ident::new("key"), block.key));
        builder.build()
    }
}
impl TryFrom<Block> for TofuTerraformAzureRMBackendBlock {
    type Error = eyre::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        if block.ident.as_str() != "backend" {
            bail!(
                "Block must use 'backend' ident, got {:?}",
                block.ident.as_str()
            );
        }
        let [label] = block.labels.as_slice() else {
            bail!(
                "Block must have exactly one label, got {}",
                block.labels.len()
            );
        };
        if label.to_string() != "azurerm" {
            bail!("Block label must equal 'azurerm'");
        }
        let resource_group_name = block
            .body
            .get_attribute("resource_group_name")
            .ok_or_eyre("Block body missing attribute: resource_group_name")?
            .value.as_str().ok_or_eyre("Attribute resource_group_name only supports string literals for conversion at this time")?.to_string();
        let storage_account_name = block
            .body
            .get_attribute("storage_account_name")
            .ok_or_eyre("Block body missing attribute: storage_account_name")?
            .value.as_str().ok_or_eyre("Attribute storage_account_name only supports string literals for conversion at this time")?.to_string();
        let container_name = block
            .body
            .get_attribute("container_name")
            .ok_or_eyre("Block body missing attribute: container_name")?
            .value.as_str().ok_or_eyre("Attribute container_name only supports string literals for conversion at this time")?.to_string();
        let subscription_id = block
            .body
            .get_attribute("subscription_id")
            .ok_or_eyre("Block body missing attribute: subscription_id")?
            .value.as_str().ok_or_eyre("Attribute subscription_id only supports string literals for conversion at this time")?.to_string();
        let key = block
            .body
            .get_attribute("key")
            .ok_or_eyre("Block body missing attribute: key")?
            .value
            .as_str()
            .ok_or_eyre("Attribute key only supports string literals for conversion at this time")?
            .to_string();
        Ok(TofuTerraformAzureRMBackendBlock {
            resource_group_name,
            storage_account_name,
            container_name,
            subscription_id,
            key,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TofuTerraformRequiredProvidersBlock(HashMap<String, TofuTerraformProviderVersionObject>);

impl TryFrom<Block> for TofuTerraformRequiredProvidersBlock {
    type Error = eyre::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        if block.ident.to_string() != "required_providers" {
            bail!("Block must use 'required_providers' ident");
        }
        let mut entries = HashMap::new();
        for attr in block.body.attributes() {
            let provider_label = &attr.key;
            let version_block = attr
                .value
                .as_object()
                .ok_or_eyre("Expected required provider value to be an object")?
                .try_into()?;
            entries.insert(provider_label.to_string(), version_block);
        }
        Ok(TofuTerraformRequiredProvidersBlock(entries))
    }
}
impl From<TofuTerraformRequiredProvidersBlock> for Block {
    fn from(value: TofuTerraformRequiredProvidersBlock) -> Self {
        let mut builder = Block::builder(Ident::new("required_providers"));
        for (provider, body) in value.0 {
            let body: Object = body.into();
            builder = builder.attribute(Attribute::new(Decorated::new(Ident::new(provider)), body));
        }
        builder.build()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TofuTerraformProviderVersionObject {
    pub source: String,
    pub version: String,
}
impl TryFrom<&Object> for TofuTerraformProviderVersionObject {
    type Error = eyre::Error;

    fn try_from(obj: &Object) -> Result<Self, Self::Error> {
        let mut source = None;
        let mut version = None;
        for (key, value) in obj {
            match key.as_ident() {
                Some(s) => match s.to_string().as_str() {
                    "source" => {
                        if source.is_some() {
                            bail!("Duplicate key: source");
                        }
                        source = Some(
                            value
                                .expr()
                                .as_str()
                                .ok_or_eyre("Expected value to be a string literal")?
                                .to_string(),
                        );
                    }
                    "version" => {
                        if version.is_some() {
                            bail!("Duplicate key: source");
                        }
                        version = Some(
                            value
                                .expr()
                                .as_str()
                                .ok_or_eyre("Expected value to be a string literal")?
                                .to_string(),
                        );
                    }
                    x => {
                        bail!("Unexpected key: {x}");
                    }
                },
                None => {
                    bail!("Unexpected entry format, key is none\nkey={key:?}\nvalue={value:?}")
                }
            }
        }
        let source = source.ok_or_eyre("Missing source attribute")?;
        let version = version.ok_or_eyre("Missing version attribute")?;
        Ok(TofuTerraformProviderVersionObject { source, version })
    }
}
impl From<TofuTerraformProviderVersionObject> for Object {
    fn from(value: TofuTerraformProviderVersionObject) -> Self {
        let mut obj = Object::new();
        obj.insert(
            ObjectKey::Ident(Decorated::new(Ident::new("source"))),
            ObjectValue::new(value.source),
        );
        obj.insert(
            ObjectKey::Ident(Decorated::new(Ident::new("version"))),
            ObjectValue::new(value.version),
        );
        obj
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let tf = indoc! {r#"
            terraform {
                backend "azurerm" {
                    resource_group_name  = "123" 
                    storage_account_name = "456" 
                    container_name       = "789" 
                    subscription_id      = "145" 
                    key                  = "155" 
                }
                required_providers {
                    azurerm = {
                        source = "hashicorp/azurerm"
                        version = ">=4.18.0"
                    }
                }
            }
        "#};
        let tf = tf.parse::<Body>()?.into_blocks().next().unwrap();
        let x: TofuTerraformBlock = tf.try_into()?;
        dbg!(&x);
        assert_eq!(
            x,
            TofuTerraformBlock {
                backend: Some(TofuTerraformBackendBlock::AzureRM(
                    TofuTerraformAzureRMBackendBlock {
                        resource_group_name: "123".to_string(),
                        storage_account_name: "456".to_string(),
                        container_name: "789".to_string(),
                        subscription_id: "145".to_string(),
                        key: "155".to_string()
                    }
                )),
                required_providers: Some(TofuTerraformRequiredProvidersBlock(
                    [(
                        "azurerm".to_string(),
                        TofuTerraformProviderVersionObject {
                            source: "hashicorp/azurerm".to_string(),
                            version: ">=4.18.0".to_string()
                        }
                    )]
                    .into()
                )),
                other: vec![],
            }
        );
        Ok(())
    }
}
