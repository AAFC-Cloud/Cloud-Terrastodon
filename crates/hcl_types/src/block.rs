use crate::data_block::HclDataBlock;
use crate::prelude::HclImportBlock;
use crate::prelude::HclProviderBlock;
use crate::prelude::HclResourceBlock;
use crate::prelude::TerraformBlock;
use eyre::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use itertools::Itertools;
use strum::EnumDiscriminants;

#[derive(Debug, Clone, Eq, PartialEq, EnumDiscriminants)]
#[strum_discriminants(name(HclBlockKind))]
pub enum HclBlock {
    Terraform(TerraformBlock),
    Provider(HclProviderBlock),
    Import(HclImportBlock),
    Resource(HclResourceBlock),
    Data(HclDataBlock),
    Other(Block),
}
impl HclBlock {
    pub fn kind(&self) -> HclBlockKind {
        match self {
            HclBlock::Terraform(_) => HclBlockKind::Terraform,
            HclBlock::Provider(_) => HclBlockKind::Provider,
            HclBlock::Import(_) => HclBlockKind::Import,
            HclBlock::Resource(_) => HclBlockKind::Resource,
            HclBlock::Data(_) => HclBlockKind::Data,
            HclBlock::Other(_) => HclBlockKind::Other,
        }
    }
}
impl From<HclProviderBlock> for HclBlock {
    fn from(value: HclProviderBlock) -> Self {
        HclBlock::Provider(value)
    }
}
impl From<HclImportBlock> for HclBlock {
    fn from(value: HclImportBlock) -> Self {
        HclBlock::Import(value)
    }
}
impl From<TerraformBlock> for HclBlock {
    fn from(value: TerraformBlock) -> Self {
        HclBlock::Terraform(value)
    }
}
impl TryFrom<Block> for HclBlock {
    type Error = eyre::Error;
    fn try_from(block: Block) -> Result<Self> {
        Ok(match block.ident.as_str() {
            "import" => {
                let block = HclImportBlock::try_from(block)?;
                HclBlock::Import(block)
            }
            "provider" => {
                let block = HclProviderBlock::try_from(block)?;
                HclBlock::Provider(block)
            }
            "terraform" => {
                let block = TerraformBlock::try_from(block)?;
                HclBlock::Terraform(block)
            }
            "resource" => {
                let block = HclResourceBlock::try_from(block)?;
                HclBlock::Resource(block)
            }
            "data" => {
                let block = HclDataBlock::try_from(block)?;
                HclBlock::Data(block)
            }
            _ => HclBlock::Other(block),
        })
    }
}
impl TryFrom<HclBlock> for Block {
    type Error = eyre::Error;
    fn try_from(hcl_block: HclBlock) -> Result<Self> {
        Ok(match hcl_block {
            HclBlock::Import(import_block) => import_block.try_into()?,
            HclBlock::Provider(provider_block) => provider_block.into(),
            HclBlock::Terraform(terraform_block) => terraform_block.into(),
            HclBlock::Resource(block) => block.into(),
            HclBlock::Data(block) => block.into(),
            HclBlock::Other(block) => block,
        })
    }
}
impl std::fmt::Display for HclBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HclBlock::Terraform(terraform) => f.write_fmt(format_args!(
                "Terraform block - backend={}, required_providers={}, other={}",
                terraform.backend.is_some(),
                terraform.required_providers.is_some(),
                terraform.other.len()
            )),
            HclBlock::Provider(provider) => match provider.alias() {
                Some(alias) => f.write_fmt(format_args!(
                    "provider {} - alias={}",
                    provider.provider_kind(),
                    alias
                )),
                None => f.write_fmt(format_args!("provider {}", provider.provider_kind())),
            },
            HclBlock::Import(import_block) => f.write_fmt(format_args!(
                "import to {} from {}",
                import_block.to, import_block.id
            )),
            HclBlock::Resource(block) => f.write_fmt(format_args!("resource {block}",)),
            HclBlock::Data(block) => f.write_fmt(format_args!("data {block}",)),
            HclBlock::Other(block) => {
                if (block.ident.to_string() == "data" || block.ident.to_string() == "resource")
                    && let Some(name) = block
                        .body
                        .get_attribute("display_name")
                        .or_else(|| block.body.get_attribute("name"))
                    && block
                        .labels
                        .get(1)
                        .map(|label| label.to_string())
                        .filter(|label| Some(label.as_str()) != name.value.as_str())
                        .is_some()
                {
                    f.write_fmt(format_args!(
                        "{} {} - {} = {}",
                        block.ident,
                        block.labels.iter().map(|x| x.to_string()).join(" "),
                        name.key,
                        name.value
                    ))
                } else {
                    f.write_fmt(format_args!(
                        "{} {}",
                        block.ident,
                        block.labels.iter().map(|x| x.to_string()).join(".")
                    ))
                }
            }
        }
    }
}

pub trait IntoHclBlocks {
    fn try_into_hcl_blocks(self) -> Result<Vec<HclBlock>>;
}
impl IntoHclBlocks for Body {
    fn try_into_hcl_blocks(self) -> Result<Vec<HclBlock>> {
        let mut rtn = Vec::new();
        for block in self.into_blocks() {
            rtn.push(block.try_into()?);
        }
        Ok(rtn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::HclProviderReference;
    use crate::prelude::ProviderKind;
    use crate::prelude::ResourceBlockReference;

    #[test]
    fn it_works() -> Result<()> {
        let body: Body = r#"
        import {
          id = "guh"
          to = providername_resourcekind.name
        }
        "#
        .parse()?;
        let mut blocks = body.try_into_hcl_blocks()?;
        assert_eq!(blocks.len(), 1);
        let block = blocks.pop().unwrap();
        assert!(matches!(
            block,
            HclBlock::Import(HclImportBlock {
                provider: HclProviderReference::Inherited,
                id,
                to: ResourceBlockReference::Other { provider, kind, name }
            }) if id == "guh" && provider == ProviderKind::Other("providername".to_string()) && kind == "resourcekind" && name == "name"
        ));

        Ok(())
    }
}
