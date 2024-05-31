use crate::prelude::AsTofuString;
use crate::prelude::TofuResourceReference;
use crate::providers::TofuProviderReference;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use hcl::edit::structure::Block;
use indoc::formatdoc;
use std::collections::HashSet;

#[derive(Debug)]
pub struct TofuImportBlock {
    pub provider: TofuProviderReference,
    pub id: String,
    //     pub id: ScopeImpl,
    pub to: TofuResourceReference,
}
impl TofuImportBlock {
    pub fn using_provider_alias(&self, provider: TofuProviderReference) -> TofuImportBlock {
        TofuImportBlock {
            provider,
            id: self.id.clone(),
            to: self.to.clone(),
        }
    }
}

impl TryFrom<TofuImportBlock> for Block {
    type Error = anyhow::Error;

    fn try_from(value: TofuImportBlock) -> Result<Self> {
        let body = value
            .as_tofu_string()
            .parse::<hcl::edit::structure::Body>()
            .context("should be valid body")?;
        body.into_blocks()
            .next()
            .ok_or(anyhow!("parsed body should contain the import block"))
    }
}

impl AsTofuString for TofuImportBlock {
    fn as_tofu_string(&self) -> String {
        let provider = match &self.provider {
            TofuProviderReference::Alias { kind, name } => format!("\n    provider = {kind}.{name}"),
            TofuProviderReference::Default { kind: None } => "".to_string(),
            TofuProviderReference::Default { kind: Some(kind) } => format!("\n    provider = {kind}"),
        };
        formatdoc! {
            r#"
                import {{{}
                    id = "{}"
                    to = {}
                }}
            "#,
            provider,
            self.id,
            self.to
        }
    }
}

impl AsTofuString for Vec<TofuImportBlock> {
    fn as_tofu_string(&self) -> String {
        let mut rtn = String::new();
        let mut seen = HashSet::new();
        for import in self.iter() {
            if seen.contains(&import.id) {
                continue;
            } else {
                seen.insert(&import.id);
            }
            rtn.push_str(import.as_tofu_string().as_str());
            rtn.push('\n');
        }

        rtn
    }
}
