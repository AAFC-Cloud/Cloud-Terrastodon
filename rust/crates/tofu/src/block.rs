use hcl::edit::structure::Block;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuProviderBlock;

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
impl From<Block> for TofuBlock {
    fn from(value: Block) -> Self {
        TofuBlock::Other(value)
    }
}