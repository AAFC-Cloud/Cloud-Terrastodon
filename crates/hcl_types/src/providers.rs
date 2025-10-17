use eyre::OptionExt;
use eyre::bail;
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
pub enum ProviderKind {
    AzureRM,
    AzureAD,
    AzureDevOps,
    Other(String),
}
impl ProviderKind {
    pub fn provider_prefix(&self) -> &str {
        match self {
            ProviderKind::AzureRM => "azurerm",
            ProviderKind::AzureAD => "azuread",
            ProviderKind::AzureDevOps => "azuredevops",
            ProviderKind::Other(s) => s.as_str(),
        }
    }
    pub fn well_known_variants() -> [ProviderKind; 3] {
        [
            ProviderKind::AzureRM,
            ProviderKind::AzureAD,
            ProviderKind::AzureDevOps,
        ]
    }
}
impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.provider_prefix())
    }
}
impl FromStr for ProviderKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for char in s.chars() {
            if !char.is_alphabetic() {
                bail!("Invalid character {char} parsing provider kind {s}");
            }
        }
        for kind in ProviderKind::well_known_variants() {
            if kind.provider_prefix() == s {
                return Ok(kind);
            }
        }
        Ok(ProviderKind::Other(s.to_owned()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HCLProviderReference {
    Alias { kind: ProviderKind, name: String },
    Default { kind: ProviderKind },
    Inherited,
}
impl HCLProviderReference {
    pub fn kind(&self) -> Option<&ProviderKind> {
        match self {
            HCLProviderReference::Alias { kind, .. } => Some(kind),
            HCLProviderReference::Default { kind } => Some(kind),
            HCLProviderReference::Inherited => None,
        }
    }
    pub fn try_as_expression(&self) -> Option<Result<Expression, parser::Error>> {
        match self {
            HCLProviderReference::Alias { kind, name } => {
                Some(format!("{kind}.{name}").parse::<Expression>())
            }
            HCLProviderReference::Default { kind } => Some(format!("{kind}").parse::<Expression>()),
            HCLProviderReference::Inherited => None,
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
impl TryFrom<Block> for HCLProviderReference {
    type Error = eyre::Error;
    // TODO: optimize this to take a reference instead of cloning the block; make it a custom trait
    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let Some(provider_attr) = block.body.get_attribute("provider") else {
            match block.ident.to_string().as_str() {
                "resource" | "data" | "import" => {
                    return Ok(HCLProviderReference::Inherited);
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
        let provider_kind: ProviderKind = provider_kind.parse()?;
        let provider_alias =
            traversal
                .operators
                .first()
                .and_then(|operator| match operator.value() {
                    TraversalOperator::GetAttr(attr) => Some(attr.to_string()),
                    _ => None,
                });
        Ok(match provider_alias {
            Some(alias) => HCLProviderReference::Alias {
                kind: provider_kind,
                name: alias,
            },
            None => HCLProviderReference::Default {
                kind: provider_kind,
            },
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HCLProviderBlock {
    AzureRM {
        alias: Option<String>,
        subscription_id: Option<String>,
    },
    AzureAD {
        alias: Option<String>,
    },
    AzureDevOps {
        alias: Option<String>,
        org_service_url: String,
    },
    Other {
        kind: String,
        alias: Option<String>,
    },
}
impl HCLProviderBlock {
    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            HCLProviderBlock::AzureRM { .. } => ProviderKind::AzureRM,
            HCLProviderBlock::AzureAD { .. } => ProviderKind::AzureAD,
            HCLProviderBlock::AzureDevOps { .. } => ProviderKind::AzureDevOps,
            HCLProviderBlock::Other { kind, .. } => ProviderKind::Other(kind.to_owned()),
        }
    }
    pub fn alias(&self) -> Option<&String> {
        match self {
            HCLProviderBlock::AzureRM { alias, .. } => alias.as_ref(),
            HCLProviderBlock::AzureAD { alias, .. } => alias.as_ref(),
            HCLProviderBlock::AzureDevOps { alias, .. } => alias.as_ref(),
            HCLProviderBlock::Other { alias, .. } => alias.as_ref(),
        }
    }
    pub fn as_reference(&self) -> HCLProviderReference {
        match self.alias().cloned() {
            Some(name) => HCLProviderReference::Alias {
                kind: self.provider_kind(),
                name,
            },
            None => HCLProviderReference::Default {
                kind: self.provider_kind(),
            },
        }
    }
}
impl From<HCLProviderBlock> for Block {
    fn from(provider: HCLProviderBlock) -> Self {
        // Create block with label
        let mut builder = Block::builder(Ident::new("provider"))
            .label(provider.provider_kind().provider_prefix());

        // Add alias if present
        if let Some(alias) = provider.alias() {
            builder = builder.attribute(Attribute::new(Ident::new("alias"), alias.to_owned()));
        }

        // Kind-specific configuration
        match provider {
            HCLProviderBlock::AzureRM {
                alias: _,
                subscription_id,
            } => {
                builder = builder
                    .block(Block::builder(Ident::new("features")).build())
                    .attribute(Attribute::new(
                        Ident::new("resource_provider_registrations"),
                        "none",
                    ));
                if let Some(subscription_id) = subscription_id {
                    builder = builder.attribute(Attribute::new(
                        Ident::new("subscription_id"),
                        subscription_id,
                    ));
                }
            }
            HCLProviderBlock::AzureAD { alias: _ } => {}
            HCLProviderBlock::AzureDevOps {
                alias: _,
                org_service_url,
            } => {
                builder = builder.attribute(Attribute::new(
                    Ident::new("org_service_url"),
                    org_service_url,
                ));
            }
            HCLProviderBlock::Other { kind: _, alias: _ } => {}
        }

        // Return
        builder.build()
    }
}
impl TryFrom<Block> for HCLProviderBlock {
    type Error = eyre::Error;

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
        let provider_kind = ProviderKind::from_str(label)?;
        let provider_block = match provider_kind {
            ProviderKind::AzureRM => HCLProviderBlock::AzureRM {
                alias,
                subscription_id: block
                    .body
                    .get_attribute("subscription_id")
                    .and_then(|attr| attr.value.as_str())
                    .map(|s| s.to_owned()),
            },
            ProviderKind::AzureAD => HCLProviderBlock::AzureAD { alias },
            ProviderKind::AzureDevOps => {
                let org_service_url = block
                    .body
                    .get_attribute("org_service_url")
                    .and_then(|attr| attr.value.as_str())
                    .map(|s| s.to_owned())
                    .ok_or_eyre("Expected org_service_url in devops block")?;
                HCLProviderBlock::AzureDevOps {
                    alias,
                    org_service_url,
                }
            }
            ProviderKind::Other(kind) => HCLProviderBlock::Other { kind, alias },
        };

        Ok(provider_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::AsHCLString;
    use std::collections::HashSet;

    #[test]
    fn it_works1() -> eyre::Result<()> {
        let provider = HCLProviderBlock::AzureRM {
            alias: Some("bruh".to_string()),
            subscription_id: Some("abc".to_string()),
        };
        let block: Block = provider.clone().into();
        println!("{}", block.as_hcl_string());
        let back: HCLProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn it_works2() -> eyre::Result<()> {
        let provider = HCLProviderBlock::AzureAD { alias: None };
        let block: Block = provider.clone().into();
        println!("{}", block.as_hcl_string());
        let back: HCLProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn it_works3() -> eyre::Result<()> {
        let provider = HCLProviderBlock::AzureAD {
            alias: Some("yeehaw".to_string()),
        };
        let block: Block = provider.clone().into();
        println!("{}", block.as_hcl_string());
        let back: HCLProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn dedup() {
        let mut providers = HashSet::new();
        providers.insert(HCLProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(HCLProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(HCLProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        assert_eq!(providers.len(), 1);
        providers.insert(HCLProviderBlock::AzureRM {
            alias: Some("bruh".to_owned()),
            subscription_id: None,
        });
        assert_eq!(providers.len(), 2);
    }
}
