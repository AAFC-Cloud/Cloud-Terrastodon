use anyhow::Result;
use hcl::edit::structure::Body;
use hcl::edit::structure::IntoBlocks;

pub trait AsTofuString {
    fn as_tofu_string(&self) -> String;
}
impl AsTofuString for String {
    fn as_tofu_string(&self) -> String {
        self.to_owned()
    }
}
impl AsTofuString for &str {
    fn as_tofu_string(&self) -> String {
        self.to_string()
    }
}

pub trait Sanitizable {
    fn sanitize(&self) -> String;
}

impl<T: AsRef<str>> Sanitizable for T {
    fn sanitize(&self) -> String {
        self.as_ref()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .skip_while(|c| *c == '_')
            .collect()
    }
}

pub trait TryAsTofuBlocks {
    fn try_as_tofu_blocks(&self) -> Result<IntoBlocks>;
}
impl<T: AsTofuString> TryAsTofuBlocks for T {
    fn try_as_tofu_blocks(&self) -> Result<IntoBlocks> {
        Ok(self.as_tofu_string().parse::<Body>()?.into_blocks())
    }
}
