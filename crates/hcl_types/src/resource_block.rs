use crate::prelude::AzureAdResourceBlockKind;
use crate::prelude::AzureDevOpsResourceBlockKind;
use crate::prelude::AzureRmResourceBlockKind;
use crate::prelude::ProviderKind;
use crate::prelude::ResourceBlockReference;
use crate::prelude::ResourceBlockResourceKind;
use crate::resource_block_kind_other::OtherResourceBlockKind;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl_primitives::Ident;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HclResourceBlock {
    AzureRM {
        kind: AzureRmResourceBlockKind,
        name: String,
        body: Body, // this can be specialized later
    },
    AzureAD {
        kind: AzureAdResourceBlockKind,
        name: String,
        body: Body, // this can be specialized later
    },
    AzureDevOps {
        kind: AzureDevOpsResourceBlockKind,
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
impl std::fmt::Display for HclResourceBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "resource \"{provider}_{kind}\" \"{name}\"",
            provider = self.provider_kind().provider_prefix(),
            kind = self.resource_kind(),
            name = self.resource_name(),
        )
    }
}
impl TryFrom<Block> for HclResourceBlock {
    type Error = eyre::Error;

    fn try_from(value: Block) -> Result<Self, Self::Error> {
        if value.ident.as_str() != "resource" {
            eyre::bail!("Expected resource block, got {}", value.ident);
        }
        let [kind, name] = value.labels.as_slice() else {
            eyre::bail!(
                "Expected resource block to have exactly two labels, got {}",
                value.labels.len()
            );
        };
        let kind = kind.parse::<ResourceBlockResourceKind>()?;
        let name = name.as_str().to_owned();
        let body = value.body;
        Ok(match kind {
            ResourceBlockResourceKind::AzureRM(kind) => {
                HclResourceBlock::AzureRM { kind, name, body }
            }
            ResourceBlockResourceKind::AzureAD(kind) => {
                HclResourceBlock::AzureAD { kind, name, body }
            }
            ResourceBlockResourceKind::AzureDevOps(kind) => {
                HclResourceBlock::AzureDevOps { kind, name, body }
            }
            ResourceBlockResourceKind::Other(other) => HclResourceBlock::Other {
                provider: other.provider,
                kind: other.resource,
                name,
                body,
            },
        })
    }
}
impl From<HclResourceBlock> for Block {
    fn from(value: HclResourceBlock) -> Self {
        let ident = Ident::new("resource");
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
impl From<HclResourceBlock> for Structure {
    fn from(value: HclResourceBlock) -> Self {
        Block::from(value).into()
    }
}
impl HclResourceBlock {
    pub fn resource_kind(&self) -> ResourceBlockResourceKind {
        match self {
            HclResourceBlock::AzureRM { kind, .. } => {
                ResourceBlockResourceKind::AzureRM(kind.clone())
            }
            HclResourceBlock::AzureAD { kind, .. } => {
                ResourceBlockResourceKind::AzureAD(kind.clone())
            }
            HclResourceBlock::AzureDevOps { kind, .. } => {
                ResourceBlockResourceKind::AzureDevOps(kind.clone())
            }
            HclResourceBlock::Other { provider, kind, .. } => {
                ResourceBlockResourceKind::Other(OtherResourceBlockKind {
                    provider: provider.to_owned(),
                    resource: kind.to_owned(),
                })
            }
        }
    }
    pub fn resource_name(&self) -> &str {
        match self {
            HclResourceBlock::AzureRM { name, .. } => name.as_str(),
            HclResourceBlock::AzureAD { name, .. } => name.as_str(),
            HclResourceBlock::AzureDevOps { name, .. } => name.as_str(),
            HclResourceBlock::Other { name, .. } => name.as_str(),
        }
    }
    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            HclResourceBlock::AzureRM { .. } => ProviderKind::AzureRM,
            HclResourceBlock::AzureAD { .. } => ProviderKind::AzureAD,
            HclResourceBlock::AzureDevOps { .. } => ProviderKind::AzureDevOps,
            HclResourceBlock::Other { provider, .. } => provider.to_owned(),
        }
    }
    pub fn as_resource_block_reference(&self) -> ResourceBlockReference {
        match self {
            HclResourceBlock::AzureRM { kind, name, .. } => ResourceBlockReference::AzureRM {
                kind: kind.to_owned(),
                name: name.to_owned(),
            },
            HclResourceBlock::AzureAD { kind, name, .. } => ResourceBlockReference::AzureAD {
                kind: kind.to_owned(),
                name: name.to_owned(),
            },
            HclResourceBlock::AzureDevOps { kind, name, .. } => {
                ResourceBlockReference::AzureDevOps {
                    kind: kind.to_owned(),
                    name: name.to_owned(),
                }
            }
            HclResourceBlock::Other {
                provider,
                kind,
                name,
                ..
            } => ResourceBlockReference::Other {
                provider: provider.to_owned(),
                kind: kind.to_owned(),
                name: name.to_owned(),
            },
        }
    }
    pub fn into_body(self) -> Body {
        match self {
            HclResourceBlock::AzureRM { body, .. } => body,
            HclResourceBlock::AzureAD { body, .. } => body,
            HclResourceBlock::AzureDevOps { body, .. } => body,
            HclResourceBlock::Other { body, .. } => body,
        }
    }

    pub fn body(&self) -> &Body {
        match self {
            HclResourceBlock::AzureRM { body, .. } => body,
            HclResourceBlock::AzureAD { body, .. } => body,
            HclResourceBlock::AzureDevOps { body, .. } => body,
            HclResourceBlock::Other { body, .. } => body,
        }
    }
}
