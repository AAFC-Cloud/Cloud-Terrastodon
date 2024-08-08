use std::str::FromStr;

use crate::prelude::AsTofuString;
use crate::prelude::TofuProviderKind;
use crate::prelude::TofuResourceReference;
use crate::providers::TofuProviderReference;
use anyhow::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl_primitives::Ident;
use indoc::formatdoc;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
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

impl TryFrom<Block> for TofuImportBlock {
    type Error = anyhow::Error;

    fn try_from(mut block: Block) -> Result<Self, Self::Error> {
        let ident = block.ident.as_str();
        if ident != "import" {
            bail!("Block isn't an import block: {block:?}")
        }
        let provider = match block.body.get_attribute("provider") {
            Some(attrib) => {
                let provider: TofuProviderReference = match &attrib.value {
                    Expression::Variable(v) => TofuProviderReference::Default {
                        kind: TofuProviderKind::from_str(v.as_str())?,
                    },
                    Expression::Traversal(traversal) => {
                        // Determine kind
                        let Expression::Variable(ref v) = traversal.expr else {
                            bail!("Expected tofu provider traversal to be a variable, got {:?} from {:?}", traversal.expr, block);
                        };
                        let kind = TofuProviderKind::from_str(v.as_str())?;

                        // Determine alias
                        let Some(alias) = traversal.operators.first() else {
                            bail!("Expected tofu provider traversal to have an alias, failed to find on {:?} from {:?}", traversal, block);
                        };
                        let TraversalOperator::GetAttr(ref alias) = alias.value() else {
                            bail!("Expected tofu provider traversal alias to be a getter, instead got {:?} from {:?}", alias.value(), block);
                        };
                        let alias = alias.as_str();

                        TofuProviderReference::Alias {
                            kind,
                            name: alias.to_owned(),
                        }
                    }
                    _ => {
                        bail!("Unable to understand attribute \"provider\" in import block, expected a traversal, got {:?} from {:?}", attrib.value, block);
                    }
                };
                provider
            }
            None => TofuProviderReference::Inherited,
        };

        // Get ID attrib
        let Some(id) = block.body.get_attribute("id") else {
            bail!("Missing attribute \"id\" interpreting block as an import block: {:?}", block);
        };
        let Some(id) = id.value.as_str() else {
            bail!("Failed to interpret id={id:?} as a string literal");
        };
        let id = id.to_owned();

        // Get TO attrib
        let Some(to) = block.body.remove_attribute("to") else {
            bail!("Missing attribute \"to\" interpreting block as an import block: {:?}", block);
        };
        let to: TofuResourceReference = TofuResourceReference::try_from(to.value)?;
       

        Ok(TofuImportBlock {
            provider,
            id,
            to,
        })
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
    #[test]
    fn import_block_conversion1() -> anyhow::Result<()> {
        let content = r#"
            import {
                provider = azurerm.abc
                id = "123"
                to = azurerm_test.one_two_three
            }
        "#;
        let hcl_block = content.try_as_tofu_blocks()?.next().unwrap();
        let our_block: TofuImportBlock = hcl_block.try_into()?;
        let expected = TofuImportBlock {
            provider: TofuProviderReference::Alias {
                kind: TofuProviderKind::AzureRM,
                name: "abc".to_string(),
            },
            id: "123".to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::Other("test".to_string()),
                name: "one_two_three".to_string(),
            },
        };
        assert_eq!(our_block, expected);
        Ok(())
    }
    #[test]
    fn import_block_conversion2() -> anyhow::Result<()> {
        let content = r#"
            import {
                id = "123"
                to = azurerm_test.one_two_three
            }
        "#;
        let hcl_block = content.try_as_tofu_blocks()?.next().unwrap();
        let our_block: TofuImportBlock = hcl_block.try_into()?;
        let expected = TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: "123".to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::Other("test".to_string()),
                name: "one_two_three".to_string(),
            },
        };
        assert_eq!(our_block, expected);
        Ok(())
    }
    #[test]
    fn import_block_conversion3() -> anyhow::Result<()> {
        let content = r#"
            import {
                provider = azurerm
                id = "123"
                to = azurerm_test.one_two_three
            }
        "#;
        let hcl_block = content.try_as_tofu_blocks()?.next().unwrap();
        let our_block: TofuImportBlock = hcl_block.try_into()?;
        let expected = TofuImportBlock {
            provider: TofuProviderReference::Default {
                kind: TofuProviderKind::AzureRM,
            },
            id: "123".to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::Other("test".to_string()),
                name: "one_two_three".to_string(),
            },
        };
        assert_eq!(our_block, expected);
        Ok(())
    }
}
