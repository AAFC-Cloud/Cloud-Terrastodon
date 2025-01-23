use std::collections::HashSet;

use eyre::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::IntoBlocks;
use hcl_primitives::ident::is_id_continue;
use hcl_primitives::ident::is_id_start;

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
        let mut rtn: String = self
            .as_ref()
            .chars()
            // .flat_map(|c| unidecode_char(c).chars())
            .enumerate()
            .map(|(i, c)| {
                if i == 0 && is_id_start(c) || i > 0 && is_id_continue(c) {
                    c
                } else {
                    '_'
                }
            })
            .skip_while(|c| *c == '_')
            .collect();
        match rtn.chars().next() {
            Some(x) if !is_id_start(x) => {
                rtn.insert_str(0, "ZZZ_");
            }
            _ => {}
        };
        rtn
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

impl<T> AsTofuString for Vec<T>
where
    T: AsTofuString,
{
    fn as_tofu_string(&self) -> String {
        let mut rtn = String::new();
        for v in self.iter() {
            rtn.push_str(v.as_tofu_string().as_str());
            rtn.push('\n');
        }
        rtn
    }
}
impl<T> AsTofuString for HashSet<T>
where
    T: AsTofuString,
{
    fn as_tofu_string(&self) -> String {
        let mut rtn = String::new();
        for v in self.iter() {
            rtn.push_str(v.as_tofu_string().as_str());
            rtn.push('\n');
        }
        rtn
    }
}

impl AsTofuString for Block {
    fn as_tofu_string(&self) -> String {
        Body::builder().block(self.clone()).build().to_string()
    }
}

impl AsTofuString for Body {
    fn as_tofu_string(&self) -> String {
        self.to_string()
    }
}
