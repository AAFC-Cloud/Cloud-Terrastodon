use std::str::FromStr;

use crate::prelude::AsHCLString;
use crate::prelude::ProviderKind;
use crate::prelude::ResourceBlockReference;
use crate::providers::HCLProviderReference;
use eyre::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl_primitives::Ident;
use indoc::formatdoc;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct HCLImportBlock {
    pub provider: HCLProviderReference,
    //     pub id: ScopeImpl,
    pub id: String,
    pub to: ResourceBlockReference,
}

impl AsHCLString for HCLImportBlock {
    fn as_hcl_string(&self) -> String {
        let provider = match &self.provider {
            HCLProviderReference::Alias { kind, name } => {
                format!("\n    provider = {kind}.{name}")
            }
            HCLProviderReference::Inherited => "".to_string(),
            HCLProviderReference::Default { kind } => {
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

impl TryFrom<HCLImportBlock> for Block {
    type Error = parser::Error;

    fn try_from(value: HCLImportBlock) -> std::prelude::v1::Result<Self, Self::Error> {
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

impl TryFrom<Block> for HCLImportBlock {
    type Error = eyre::Error;

    fn try_from(mut block: Block) -> Result<Self, Self::Error> {
        let ident = block.ident.as_str();
        if ident != "import" {
            bail!("Block isn't an import block: {block:?}")
        }
        let provider = match block.body.get_attribute("provider") {
            Some(attrib) => {
                let provider: HCLProviderReference = match &attrib.value {
                    Expression::Variable(v) => HCLProviderReference::Default {
                        kind: ProviderKind::from_str(v.as_str())?,
                    },
                    Expression::Traversal(traversal) => {
                        // Determine kind
                        let Expression::Variable(ref v) = traversal.expr else {
                            bail!(
                                "Expected Terraform provider traversal to be a variable, got {:?} from {:?}",
                                traversal.expr,
                                block
                            );
                        };
                        let kind = ProviderKind::from_str(v.as_str())?;

                        // Determine alias
                        let Some(alias) = traversal.operators.first() else {
                            bail!(
                                "Expected Terraform provider traversal to have an alias, failed to find on {:?} from {:?}",
                                traversal,
                                block
                            );
                        };
                        let TraversalOperator::GetAttr(alias) = alias.value() else {
                            bail!(
                                "Expected Terraform provider traversal alias to be a getter, instead got {:?} from {:?}",
                                alias.value(),
                                block
                            );
                        };
                        let alias = alias.as_str();

                        HCLProviderReference::Alias {
                            kind,
                            name: alias.to_owned(),
                        }
                    }
                    _ => {
                        bail!(
                            "Unable to understand attribute \"provider\" in import block, expected a traversal, got {:?} from {:?}",
                            attrib.value,
                            block
                        );
                    }
                };
                provider
            }
            None => HCLProviderReference::Inherited,
        };

        // Get ID attrib
        let Some(id) = block.body.get_attribute("id") else {
            bail!(
                "Missing attribute \"id\" interpreting block as an import block: {:?}",
                block
            );
        };
        let Some(id) = id.value.as_str() else {
            bail!("Failed to interpret id={id:?} as a string literal");
        };
        let id = id.to_owned();

        // Get TO attrib
        let Some(to) = block.body.remove_attribute("to") else {
            bail!(
                "Missing attribute \"to\" interpreting block as an import block: {:?}",
                block
            );
        };
        let to: ResourceBlockReference = ResourceBlockReference::try_from(to.value)?;

        Ok(HCLImportBlock { provider, id, to })
    }
}

// pub trait TryIntoHCLImportBlock {
//     fn try_as_import_block(self) -> eyre::Result<HCLImportBlock>;
// }
// impl<T, E> TryIntoHCLImportBlock for T
// where
//     T: TryInto<HCLImportBlock, Error = E>,
//     E: std::error::Error + Send + Sync + 'static,
// {
//     fn try_as_import_block(self) -> eyre::Result<HCLImportBlock> {
//         Ok(self.try_into()?)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::prelude::TryAsHCLBlocks;
    use crate::providers::ProviderKind;
    use crate::resources::AzureRMResourceBlockKind;

    use super::*;

    #[test]
    fn conversaion_parity1() -> eyre::Result<()> {
        let import = HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: "abc123".to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::ResourceGroup,
                name: "my_rg".to_string(),
            },
        };
        println!("{}", import.as_hcl_string());
        let block_from_str = import.try_as_hcl_blocks()?.into_iter().next().unwrap();
        let block_from_into: Block = import.try_into()?;
        assert_eq!(block_from_into, block_from_str);
        Ok(())
    }
    #[test]
    fn conversaion_parity2() -> eyre::Result<()> {
        let import = HCLImportBlock {
            provider: HCLProviderReference::Alias {
                kind: ProviderKind::AzureRM,
                name: "prod".to_string(),
            },
            id: "abc123".to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::ResourceGroup,
                name: "my_rg".to_string(),
            },
        };
        println!("{}", import.as_hcl_string());
        let block_from_str = import.try_as_hcl_blocks()?.into_iter().next().unwrap();
        let block_from_into: Block = import.try_into()?;
        assert_eq!(block_from_into, block_from_str);
        Ok(())
    }
    #[test]
    fn import_block_conversion1() -> eyre::Result<()> {
        let content = r#"
            import {
                provider = azurerm.abc
                id = "123"
                to = azurerm_test.one_two_three
            }
        "#;
        let hcl_block = content.try_as_hcl_blocks()?.next().unwrap();
        let our_block: HCLImportBlock = hcl_block.try_into()?;
        let expected = HCLImportBlock {
            provider: HCLProviderReference::Alias {
                kind: ProviderKind::AzureRM,
                name: "abc".to_string(),
            },
            id: "123".to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::Other("test".to_string()),
                name: "one_two_three".to_string(),
            },
        };
        assert_eq!(our_block, expected);
        Ok(())
    }
    #[test]
    fn import_block_conversion2() -> eyre::Result<()> {
        let content = r#"
            import {
                id = "123"
                to = azurerm_test.one_two_three
            }
        "#;
        let hcl_block = content.try_as_hcl_blocks()?.next().unwrap();
        let our_block: HCLImportBlock = hcl_block.try_into()?;
        let expected = HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: "123".to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::Other("test".to_string()),
                name: "one_two_three".to_string(),
            },
        };
        assert_eq!(our_block, expected);
        Ok(())
    }
    #[test]
    fn import_block_conversion3() -> eyre::Result<()> {
        let content = r#"
            import {
                provider = azurerm
                id = "123"
                to = azurerm_test.one_two_three
            }
        "#;
        let hcl_block = content.try_as_hcl_blocks()?.next().unwrap();
        let our_block: HCLImportBlock = hcl_block.try_into()?;
        let expected = HCLImportBlock {
            provider: HCLProviderReference::Default {
                kind: ProviderKind::AzureRM,
            },
            id: "123".to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::Other("test".to_string()),
                name: "one_two_three".to_string(),
            },
        };
        assert_eq!(our_block, expected);
        Ok(())
    }
}
