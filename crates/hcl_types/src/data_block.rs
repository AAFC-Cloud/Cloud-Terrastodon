use crate::prelude::AzureAdDataBlockKind;
use crate::prelude::AzureDevOpsDataBlockKind;
use crate::prelude::AzureRmDataBlockKind;
use crate::prelude::DataBlockReference;
use crate::prelude::DataBlockResourceKind;
use crate::prelude::OtherDataBlockKind;
use crate::prelude::ProviderKind;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl_primitives::Ident;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HclDataBlock {
    AzureRM {
        kind: AzureRmDataBlockKind,
        name: String,
        body: Body, // this can be specialized later
    },
    AzureAD {
        kind: AzureAdDataBlockKind,
        name: String,
        body: Body, // this can be specialized later
    },
    AzureDevOps {
        kind: AzureDevOpsDataBlockKind,
        name: String,
        body: Body, // this can be specialized later
    },
    Other {
        provider: ProviderKind,
        kind: String,
        name: String,
        body: Body,
    },
}
impl std::fmt::Display for HclDataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "data \"{provider}_{kind}\" \"{name}\"",
            provider = self.provider_kind().provider_prefix(),
            kind = self.resource_kind(),
            name = self.resource_name(),
        )
    }
}

impl TryFrom<Block> for HclDataBlock {
    type Error = eyre::Error;

    fn try_from(value: Block) -> Result<Self, Self::Error> {
        if value.ident.as_str() != "data" {
            eyre::bail!("Expected data block, got {}", value.ident);
        }
        let [kind, name] = value.labels.as_slice() else {
            eyre::bail!(
                "Expected data block to have exactly two labels, got {}",
                value.labels.len()
            );
        };
        let kind = kind.parse::<DataBlockResourceKind>()?;
        let name = name.as_str().to_owned();
        let body = value.body;
        Ok(match kind {
            DataBlockResourceKind::AzureRM(kind) => HclDataBlock::AzureRM { kind, name, body },
            DataBlockResourceKind::AzureAD(kind) => HclDataBlock::AzureAD { kind, name, body },
            DataBlockResourceKind::AzureDevOps(kind) => {
                HclDataBlock::AzureDevOps { kind, name, body }
            }
            DataBlockResourceKind::Other(other) => HclDataBlock::Other {
                provider: other.provider,
                kind: other.resource,
                name,
                body,
            },
        })
    }
}
impl From<HclDataBlock> for Block {
    fn from(value: HclDataBlock) -> Self {
        let ident = Ident::new("data");
        let resource_kind = value.resource_kind().to_string();
        let resource_name = value.resource_name().to_owned();
        let body = value.into_body();
        Block::builder(ident)
            .label(resource_kind)
            .label(resource_name)
            .structures(body)
            .build()
    }
}
impl From<HclDataBlock> for Structure {
    fn from(value: HclDataBlock) -> Self {
        Block::from(value).into()
    }
}
impl HclDataBlock {
    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            HclDataBlock::Other { provider, .. } => provider.to_owned(),
            HclDataBlock::AzureRM { .. } => ProviderKind::AzureRM,
            HclDataBlock::AzureAD { .. } => ProviderKind::AzureAD,
            HclDataBlock::AzureDevOps { .. } => ProviderKind::AzureDevOps,
        }
    }
    pub fn resource_name(&self) -> &str {
        match self {
            HclDataBlock::Other { name, .. } => name.as_str(),
            HclDataBlock::AzureRM { name, .. } => name.as_str(),
            HclDataBlock::AzureAD { name, .. } => name.as_str(),
            HclDataBlock::AzureDevOps { name, .. } => name.as_str(),
        }
    }
    pub fn resource_kind(&self) -> DataBlockResourceKind {
        match self {
            HclDataBlock::Other { provider, kind, .. } => {
                DataBlockResourceKind::Other(OtherDataBlockKind {
                    provider: provider.to_owned(),
                    resource: kind.to_owned(),
                })
            }
            HclDataBlock::AzureRM { kind, .. } => DataBlockResourceKind::AzureRM(kind.clone()),
            HclDataBlock::AzureAD { kind, .. } => DataBlockResourceKind::AzureAD(kind.clone()),
            HclDataBlock::AzureDevOps { kind, .. } => {
                DataBlockResourceKind::AzureDevOps(kind.clone())
            }
        }
    }
    pub fn as_data_block_reference(&self) -> DataBlockReference {
        match self {
            HclDataBlock::Other {
                provider,
                kind,
                name,
                ..
            } => DataBlockReference::Other {
                provider: provider.to_owned(),
                kind: kind.to_owned(),
                name: name.to_owned(),
            },
            HclDataBlock::AzureRM { kind, name, .. } => DataBlockReference::AzureRM {
                kind: kind.clone(),
                name: name.to_owned(),
            },
            HclDataBlock::AzureAD { kind, name, .. } => DataBlockReference::Other {
                provider: ProviderKind::AzureAD,
                kind: kind.as_ref().to_owned(),
                name: name.to_owned(),
            },
            HclDataBlock::AzureDevOps { kind, name, .. } => DataBlockReference::Other {
                provider: ProviderKind::AzureDevOps,
                kind: kind.as_ref().to_owned(),
                name: name.to_owned(),
            },
        }
    }
    pub fn into_body(self) -> Body {
        match self {
            HclDataBlock::AzureRM { body, .. } => body,
            HclDataBlock::AzureAD { body, .. } => body,
            HclDataBlock::AzureDevOps { body, .. } => body,
            HclDataBlock::Other { body, .. } => body,
        }
    }
    pub fn body(&self) -> &Body {
        match self {
            HclDataBlock::AzureRM { body, .. } => body,
            HclDataBlock::AzureAD { body, .. } => body,
            HclDataBlock::AzureDevOps { body, .. } => body,
            HclDataBlock::Other { body, .. } => body,
        }
    }
}
