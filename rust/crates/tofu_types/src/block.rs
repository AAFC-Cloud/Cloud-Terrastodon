use crate::prelude::TofuImportBlock;
use crate::prelude::TofuProviderBlock;
use anyhow::Result;
use hcl::edit::structure::Block;

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
