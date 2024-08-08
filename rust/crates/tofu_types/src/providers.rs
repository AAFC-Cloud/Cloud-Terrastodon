use anyhow::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::BlockBuilder;
use hcl::edit::structure::BlockLabel;
use hcl_primitives::Ident;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderKind {
    AzureRM,
    AzureAD,
    Other(String),
}
impl TofuProviderKind {
    pub fn provider_prefix(&self) -> &str {
        match self {
            TofuProviderKind::AzureRM => "azurerm",
            TofuProviderKind::AzureAD => "azuread",
            TofuProviderKind::Other(s) => s.as_str(),
        }
    }
}
impl std::fmt::Display for TofuProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.provider_prefix())
    }
}
impl FromStr for TofuProviderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for char in s.chars() {
            if !char.is_alphabetic() {
                bail!("Invalid character {char} parsing provider kind {s}");
            }
        }
        Ok(match s {
            kind if kind == TofuProviderKind::AzureRM.provider_prefix() => {
                TofuProviderKind::AzureRM
            }
            kind if kind == TofuProviderKind::AzureAD.provider_prefix() => {
                TofuProviderKind::AzureAD
            }
            kind => TofuProviderKind::Other(kind.to_owned()),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderReference {
    Alias {
        kind: TofuProviderKind,
        name: String,
    },
    Default {
        kind: TofuProviderKind,
    },
    Inherited,
}
impl TofuProviderReference {
    pub fn kind(&self) -> Option<&TofuProviderKind> {
        match self {
            TofuProviderReference::Alias { kind, .. } => Some(kind),
            TofuProviderReference::Default { kind } => Some(kind),
            TofuProviderReference::Inherited => None,
        }
    }
    pub fn try_as_expression(&self) -> Option<Result<Expression, parser::Error>> {
        match self {
            TofuProviderReference::Alias { kind, name } => {
                Some(format!("{kind}.{name}").parse::<Expression>())
            }
            TofuProviderReference::Default { kind } => {
                Some(format!("{kind}").parse::<Expression>())
            }
            TofuProviderReference::Inherited => None,
        }
    }
    pub fn apply_to_builder(&self, builder: BlockBuilder) -> Result<BlockBuilder, parser::Error> {
        Ok(match self.try_as_expression() {
            Some(expr) => {
                let expr = expr?;
                let attr = Attribute::new(Ident::new("provider"), expr);
                builder.attribute(attr)
            }
            None => builder,
        })
    }
}
impl TryFrom<Block> for TofuProviderReference {
    type Error = anyhow::Error;
    // TODO: optimize this to take a reference instead of cloning the block; make it a custom trait
    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let Some(provider_attr) = block.body.get_attribute("provider") else {
            match block.ident.to_string().as_str() {
                "resource" | "data" | "import" => {
                    return Ok(TofuProviderReference::Inherited);
                }
                ident => {
                    bail!("Provider attribute not present in block with invalid ident {ident}");
                }
            }
        };
        let Some(traversal) = provider_attr.value.as_traversal() else {
            bail!("Provider attribute value isn't a traversal");
        };
        let Some(provider_kind) = traversal.expr.as_variable() else {
            bail!("Provider attribute traversal isn't a variable");
        };
        let provider_kind: TofuProviderKind = provider_kind.parse()?;
        let provider_alias =
            traversal
                .operators
                .first()
                .and_then(|operator| match operator.value() {
                    TraversalOperator::GetAttr(attr) => Some(attr.to_string()),
                    _ => None,
                });
        Ok(match provider_alias {
            Some(alias) => TofuProviderReference::Alias {
                kind: provider_kind,
                name: alias,
            },
            None => TofuProviderReference::Default {
                kind: provider_kind,
            },
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuProviderBlock {
    AzureRM {
        alias: Option<String>,
        subscription_id: Option<String>,
    },
    AzureAD {
        alias: Option<String>,
    },
    Other {
        kind: String,
        alias: Option<String>,
    },
}
impl TofuProviderBlock {
    pub fn provider_kind(&self) -> TofuProviderKind {
        match self {
            TofuProviderBlock::AzureRM { .. } => TofuProviderKind::AzureRM,
            TofuProviderBlock::AzureAD { .. } => TofuProviderKind::AzureAD,
            TofuProviderBlock::Other { kind, .. } => TofuProviderKind::Other(kind.to_owned()),
        }
    }
    pub fn alias(&self) -> Option<&String> {
        match self {
            TofuProviderBlock::AzureRM { alias, .. } => alias.as_ref(),
            TofuProviderBlock::AzureAD { alias, .. } => alias.as_ref(),
            TofuProviderBlock::Other { alias, .. } => alias.as_ref(),
        }
    }
}
impl From<TofuProviderBlock> for Block {
    fn from(provider: TofuProviderBlock) -> Self {
        // Create block with label
        let mut builder = Block::builder(Ident::new("provider"))
            .label(provider.provider_kind().provider_prefix());

        // Add alias if present
        if let Some(alias) = provider.alias() {
            builder = builder.attribute(Attribute::new(Ident::new("alias"), alias.to_owned()));
        }

        // Kind-specific configuration
        if let TofuProviderBlock::AzureRM {
            subscription_id, ..
        } = provider
        {
            builder = builder
                .block(Block::builder(Ident::new("features")).build())
                .attribute(Attribute::new(
                    Ident::new("skip_provider_registration"),
                    true,
                ));
            if let Some(subscription_id) = subscription_id {
                builder = builder.attribute(Attribute::new(
                    Ident::new("subscription_id"),
                    subscription_id,
                ));
            }
        }

        // Return
        builder.build()
    }
}
impl TryFrom<Block> for TofuProviderBlock {
    type Error = anyhow::Error;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        // Preconditions
        if block.ident.to_string() != "provider" {
            bail!("Block must use 'provider' ident");
        }
        if block.labels.len() != 1 {
            bail!("Block must use exactly one label")
        }
        let Some(BlockLabel::String(label)) = block.labels.first() else {
            bail!("Block label was invalid")
        };

        // Get alias if present
        let alias = block
            .body
            .get_attribute("alias")
            .and_then(|attr| attr.value.as_str())
            .map(|s| s.to_owned());

        // Kind-specific conversion
        let label = label.value();
        let provider_block = match label {
            kind if kind == TofuProviderKind::AzureRM.provider_prefix() => {
                TofuProviderBlock::AzureRM {
                    alias,
                    subscription_id: block
                        .body
                        .get_attribute("subscription_id")
                        .and_then(|attr| attr.value.as_str())
                        .map(|s| s.to_owned()),
                }
            }
            kind if kind == TofuProviderKind::AzureAD.provider_prefix() => {
                TofuProviderBlock::AzureAD { alias }
            }
            kind => TofuProviderBlock::Other {
                kind: kind.to_owned(),
                alias,
            },
        };

        Ok(provider_block)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::prelude::AsTofuString;
    use super::*;

    #[test]
    fn it_works1() -> anyhow::Result<()> {
        let provider = TofuProviderBlock::AzureRM {
            alias: Some("bruh".to_string()),
            subscription_id: Some("abc".to_string()),
        };
        let block: Block = provider.clone().try_into()?;
        println!("{}", block.as_tofu_string());
        let back: TofuProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn it_works2() -> anyhow::Result<()> {
        let provider = TofuProviderBlock::AzureAD { alias: None };
        let block: Block = provider.clone().try_into()?;
        println!("{}", block.as_tofu_string());
        let back: TofuProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn it_works3() -> anyhow::Result<()> {
        let provider = TofuProviderBlock::AzureAD {
            alias: Some("yeehaw".to_string()),
        };
        let block: Block = provider.clone().try_into()?;
        println!("{}", block.as_tofu_string());
        let back: TofuProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn dedup() {
        let mut providers = HashSet::new();
        providers.insert(TofuProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(TofuProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(TofuProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        assert_eq!(providers.len(), 1);
        providers.insert(TofuProviderBlock::AzureRM {
            alias: Some("bruh".to_owned()),
            subscription_id: None,
        });
        assert_eq!(providers.len(), 2);
    }
}
