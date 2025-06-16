use crate::prelude::HCLImportBlock;
use crate::prelude::HCLProviderBlock;
use crate::prelude::TerraformBlock;
use eyre::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use itertools::Itertools;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HCLBlock {
    Terraform(TerraformBlock),
    Provider(HCLProviderBlock),
    Import(HCLImportBlock),
    Other(Block),
}
impl From<HCLProviderBlock> for HCLBlock {
    fn from(value: HCLProviderBlock) -> Self {
        HCLBlock::Provider(value)
    }
}
impl From<HCLImportBlock> for HCLBlock {
    fn from(value: HCLImportBlock) -> Self {
        HCLBlock::Import(value)
    }
}
impl From<TerraformBlock> for HCLBlock {
    fn from(value: TerraformBlock) -> Self {
        HCLBlock::Terraform(value)
    }
}
impl TryFrom<Block> for HCLBlock {
    type Error = eyre::Error;
    fn try_from(block: Block) -> Result<Self> {
        Ok(match block.ident.as_str() {
            "import" => {
                let block = HCLImportBlock::try_from(block)?;
                HCLBlock::Import(block)
            }
            "provider" => {
                let block = HCLProviderBlock::try_from(block)?;
                HCLBlock::Provider(block)
            }
            "terraform" => {
                let block = TerraformBlock::try_from(block)?;
                HCLBlock::Terraform(block)
            }
            _ => HCLBlock::Other(block),
        })
    }
}
impl std::fmt::Display for HCLBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HCLBlock::Terraform(terraform) => f.write_fmt(format_args!(
                "Terraform block - backend={}, required_providers={}, other={}",
                terraform.backend.is_some(),
                terraform.required_providers.is_some(),
                terraform.other.len()
            )),
            HCLBlock::Provider(provider) => match provider.alias() {
                Some(alias) => f.write_fmt(format_args!(
                    "provider {} - alias={}",
                    provider.provider_kind(),
                    alias
                )),
                None => f.write_fmt(format_args!("provider {}", provider.provider_kind())),
            },
            HCLBlock::Import(import_block) => f.write_fmt(format_args!(
                "import to {} from {}",
                import_block.to, import_block.id
            )),
            HCLBlock::Other(block) => {
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

pub trait IntoHCLBlocks {
    fn try_into_hcl_blocks(self) -> Result<Vec<HCLBlock>>;
}
impl IntoHCLBlocks for Body {
    fn try_into_hcl_blocks(self) -> Result<Vec<HCLBlock>> {
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
    use crate::prelude::HCLProviderReference;
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
            HCLBlock::Import(HCLImportBlock {
                provider: HCLProviderReference::Inherited,
                id,
                to: ResourceBlockReference::Other { provider, kind, name }
            }) if id == "guh" && provider == "providername" && kind == "resourcekind" && name == "name"
        ));

        Ok(())
    }
}
