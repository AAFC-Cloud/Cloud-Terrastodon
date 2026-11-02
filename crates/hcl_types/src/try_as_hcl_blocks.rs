use crate::AsHclString;
use eyre::Result;
use hcl::edit::structure::Body;
use hcl::edit::structure::IntoBlocks;

/// Parses a value's HCL text into owned blocks.
pub trait TryAsHclBlocks {
    fn try_as_hcl_blocks(&self) -> Result<IntoBlocks>;
}

impl<T: AsHclString> TryAsHclBlocks for T {
    fn try_as_hcl_blocks(&self) -> Result<IntoBlocks> {
        Ok(self.as_hcl_string().parse::<Body>()?.into_blocks())
    }
}
