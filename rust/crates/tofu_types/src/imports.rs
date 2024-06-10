use crate::prelude::AsTofuString;
use crate::prelude::TofuResourceReference;
use crate::providers::TofuProviderReference;
use hcl::edit::expr::Expression;
use hcl::edit::parser;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl_primitives::Ident;
use indoc::formatdoc;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct TofuImportBlock {
    pub provider: TofuProviderReference,
    //     pub id: ScopeImpl,
    pub id: String,
    pub to: TofuResourceReference,
}

impl AsTofuString for TofuImportBlock {
    fn as_tofu_string(&self) -> String {
        let provider = match &self.provider {
            TofuProviderReference::Alias { kind, name } => {
                format!("\n    provider = {kind}.{name}")
            }
            TofuProviderReference::Inherited => "".to_string(),
            TofuProviderReference::Default { kind } => {
                format!("\n    provider = {kind}")
            }
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

impl TryFrom<TofuImportBlock> for Block {
    type Error = parser::Error;

    fn try_from(value: TofuImportBlock) -> std::prelude::v1::Result<Self, Self::Error> {
        let builder = Block::builder(Ident::new("import"));
        let builder = value.provider.apply_to_builder(builder)?;

        let id = value.id;
        let to: Expression = value.to.try_into()?;
        let builder = builder.attributes([
            Attribute::new(Ident::new("id"), id),
            Attribute::new(Ident::new("to"), to),
        ]);
        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::TryAsTofuBlocks;
    use crate::providers::TofuProviderKind;
    use crate::resources::TofuAzureRMResourceKind;

    use super::*;

    #[test]
    fn conversaion_parity1() -> anyhow::Result<()> {
        let import = TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: "abc123".to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::ResourceGroup,
                name: "my_rg".to_string(),
            },
        };
        println!("{}", import.as_tofu_string());
        let block_from_str = import.try_as_tofu_blocks()?.into_iter().next().unwrap();
        let block_from_into: Block = import.try_into()?;
        assert_eq!(block_from_into, block_from_str);
        Ok(())
    }
    #[test]
    fn conversaion_parity2() -> anyhow::Result<()> {
        let import = TofuImportBlock {
            provider: TofuProviderReference::Alias {
                kind: TofuProviderKind::AzureRM,
                name: "prod".to_string(),
            },
            id: "abc123".to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::ResourceGroup,
                name: "my_rg".to_string(),
            },
        };
        println!("{}", import.as_tofu_string());
        let block_from_str = import.try_as_tofu_blocks()?.into_iter().next().unwrap();
        let block_from_into: Block = import.try_into()?;
        assert_eq!(block_from_into, block_from_str);
        Ok(())
    }
}
