use crate::prelude::HclProviderReference;
use crate::prelude::ProviderKind;
use eyre::OptionExt;
use eyre::bail;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::BlockLabel;
use hcl_primitives::Ident;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HclProviderBlock {
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
impl HclProviderBlock {
    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            HclProviderBlock::AzureRM { .. } => ProviderKind::AzureRM,
            HclProviderBlock::AzureAD { .. } => ProviderKind::AzureAD,
            HclProviderBlock::AzureDevOps { .. } => ProviderKind::AzureDevOps,
            HclProviderBlock::Other { kind, .. } => ProviderKind::Other(kind.to_owned()),
        }
    }
    pub fn alias(&self) -> Option<&String> {
        match self {
            HclProviderBlock::AzureRM { alias, .. } => alias.as_ref(),
            HclProviderBlock::AzureAD { alias, .. } => alias.as_ref(),
            HclProviderBlock::AzureDevOps { alias, .. } => alias.as_ref(),
            HclProviderBlock::Other { alias, .. } => alias.as_ref(),
        }
    }
    pub fn as_reference(&self) -> HclProviderReference {
        match self.alias().cloned() {
            Some(name) => HclProviderReference::Alias {
                kind: self.provider_kind(),
                name,
            },
            None => HclProviderReference::Default {
                kind: self.provider_kind(),
            },
        }
    }
}
impl From<HclProviderBlock> for Block {
    fn from(provider: HclProviderBlock) -> Self {
        // Create block with label
        let mut builder = Block::builder(Ident::new("provider"))
            .label(provider.provider_kind().provider_prefix());

        // Add alias if present
        if let Some(alias) = provider.alias() {
            builder = builder.attribute(Attribute::new(Ident::new("alias"), alias.to_owned()));
        }

        // Kind-specific configuration
        match provider {
            HclProviderBlock::AzureRM {
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
            HclProviderBlock::AzureAD { alias: _ } => {}
            HclProviderBlock::AzureDevOps {
                alias: _,
                org_service_url,
            } => {
                builder = builder.attribute(Attribute::new(
                    Ident::new("org_service_url"),
                    org_service_url,
                ));
            }
            HclProviderBlock::Other { kind: _, alias: _ } => {}
        }

        // Return
        builder.build()
    }
}
impl TryFrom<Block> for HclProviderBlock {
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
            ProviderKind::AzureRM => HclProviderBlock::AzureRM {
                alias,
                subscription_id: block
                    .body
                    .get_attribute("subscription_id")
                    .and_then(|attr| attr.value.as_str())
                    .map(|s| s.to_owned()),
            },
            ProviderKind::AzureAD => HclProviderBlock::AzureAD { alias },
            ProviderKind::AzureDevOps => {
                let org_service_url = block
                    .body
                    .get_attribute("org_service_url")
                    .and_then(|attr| attr.value.as_str())
                    .map(|s| s.to_owned())
                    .ok_or_eyre("Expected org_service_url in devops block")?;
                HclProviderBlock::AzureDevOps {
                    alias,
                    org_service_url,
                }
            }
            ProviderKind::Other(kind) => HclProviderBlock::Other { kind, alias },
        };

        Ok(provider_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::AsHclString;
    use std::collections::HashSet;

    #[test]
    fn it_works1() -> eyre::Result<()> {
        let provider = HclProviderBlock::AzureRM {
            alias: Some("bruh".to_string()),
            subscription_id: Some("abc".to_string()),
        };
        let block: Block = provider.clone().into();
        println!("{}", block.as_hcl_string());
        let back: HclProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn it_works2() -> eyre::Result<()> {
        let provider = HclProviderBlock::AzureAD { alias: None };
        let block: Block = provider.clone().into();
        println!("{}", block.as_hcl_string());
        let back: HclProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn it_works3() -> eyre::Result<()> {
        let provider = HclProviderBlock::AzureAD {
            alias: Some("yeehaw".to_string()),
        };
        let block: Block = provider.clone().into();
        println!("{}", block.as_hcl_string());
        let back: HclProviderBlock = block.try_into()?;
        assert_eq!(provider, back);
        Ok(())
    }
    #[test]
    fn dedup() {
        let mut providers = HashSet::new();
        providers.insert(HclProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(HclProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(HclProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        assert_eq!(providers.len(), 1);
        providers.insert(HclProviderBlock::AzureRM {
            alias: Some("bruh".to_owned()),
            subscription_id: None,
        });
        assert_eq!(providers.len(), 2);
    }
}
