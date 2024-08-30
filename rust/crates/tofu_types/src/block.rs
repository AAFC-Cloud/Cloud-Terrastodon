use crate::prelude::TofuImportBlock;
use crate::prelude::TofuProviderBlock;
use anyhow::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use itertools::Itertools;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TofuBlock {
    Provider(TofuProviderBlock),
    Import(TofuImportBlock),
    Other(Block),
}
impl From<TofuProviderBlock> for TofuBlock {
    fn from(value: TofuProviderBlock) -> Self {
        TofuBlock::Provider(value)
    }
}
impl From<TofuImportBlock> for TofuBlock {
    fn from(value: TofuImportBlock) -> Self {
        TofuBlock::Import(value)
    }
}
impl TryFrom<Block> for TofuBlock {
    type Error = anyhow::Error;
    fn try_from(block: Block) -> Result<Self> {
        Ok(match block.ident.as_str() {
            "import" => {
                let block = TofuImportBlock::try_from(block)?;
                TofuBlock::Import(block)
            }
            "provider" => {
                let block = TofuProviderBlock::try_from(block)?;
                TofuBlock::Provider(block)
            }
            _ => TofuBlock::Other(block),
        })
    }
}
impl std::fmt::Display for TofuBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TofuBlock::Provider(provider) => match provider.alias() {
                Some(alias) => f.write_fmt(format_args!(
                    "provider {} - alias={}",
                    provider.provider_kind(),
                    alias
                )),
                None => f.write_fmt(format_args!("provider {}", provider.provider_kind())),
            },
            TofuBlock::Import(import_block) => f.write_fmt(format_args!(
                "import to {} from {}",
                import_block.to, import_block.id
            )),
            TofuBlock::Other(block) => {
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

pub trait IntoTofuBlocks {
    fn try_into_tofu_blocks(self) -> Result<Vec<TofuBlock>>;
}
impl IntoTofuBlocks for Body {
    fn try_into_tofu_blocks(self) -> Result<Vec<TofuBlock>> {
        let mut rtn = Vec::new();
        for block in self.into_blocks() {
            rtn.push(block.try_into()?);
        }
        Ok(rtn)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::TofuProviderReference;
    use crate::prelude::TofuResourceReference;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let body: Body = r#"
        import {
          id = "guh"
          to = providername_resourcekind.name
        }
        "#
        .parse()?;
        let mut blocks = body.try_into_tofu_blocks()?;
        assert_eq!(blocks.len(), 1);
        let block = blocks.pop().unwrap();
        assert!(matches!(
            block,
            TofuBlock::Import(TofuImportBlock {
                provider: TofuProviderReference::Inherited,
                id,
                to: TofuResourceReference::Other { provider, kind, name }
            }) if id == "guh" && provider == "providername" && kind == "resourcekind" && name == "name"
        ));

        Ok(())
    }
}
