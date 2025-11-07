use crate::prelude::AzureAdResourceBlockKind;
use crate::prelude::AzureDevOpsResourceBlockKind;
use crate::prelude::AzureRmResourceBlockKind;
use crate::prelude::ProviderKind;
use crate::prelude::ResourceBlockResourceKind;
use eyre::Context;
use eyre::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::Traversal;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use std::any::type_name;
use std::str::FromStr;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ResourceBlockReference {
    AzureRM {
        kind: AzureRmResourceBlockKind,
        name: String,
    },
    AzureAD {
        kind: AzureAdResourceBlockKind,
        name: String,
    },
    AzureDevOps {
        kind: AzureDevOpsResourceBlockKind,
        name: String,
    },
    Other {
        provider: ProviderKind,
        kind: String,
        name: String,
    },
    Raw(String), // todo: remove this variant, Other should cover this, add from_raw helper if necessary
}
impl ResourceBlockReference {
    pub fn expression_str(&self) -> String {
        format!("{}.{}", self.kind_label(), self.name_label())
    }
    pub fn provider_kind(&self) -> ProviderKind {
        match self {
            ResourceBlockReference::AzureRM { .. } => ProviderKind::AzureRM,
            ResourceBlockReference::AzureAD { .. } => ProviderKind::AzureAD,
            ResourceBlockReference::AzureDevOps { .. } => ProviderKind::AzureDevOps,
            ResourceBlockReference::Other { provider, .. } => provider.to_owned(),
            ResourceBlockReference::Raw(s) => {
                ProviderKind::Other(s.split_once('_').map(|x| x.0).unwrap_or(s).to_owned())
            }
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            ResourceBlockReference::AzureRM { kind, .. } => kind.as_ref(),
            ResourceBlockReference::AzureAD { kind, .. } => kind.as_ref(),
            ResourceBlockReference::AzureDevOps { kind, .. } => kind.as_ref(),
            ResourceBlockReference::Other { kind, .. } => kind.as_ref(),
            ResourceBlockReference::Raw(value) => {
                value.split_once(".").map(|pair| pair.0).unwrap_or(value)
            }
        }
    }
    pub fn kind_label(&self) -> String {
        match self {
            Self::AzureRM { kind, .. } => format!(
                "{}_{}",
                ProviderKind::AzureRM.provider_prefix(),
                kind.as_ref()
            ),
            Self::AzureAD { kind, .. } => format!(
                "{}_{}",
                ProviderKind::AzureAD.provider_prefix(),
                kind.as_ref()
            ),
            Self::AzureDevOps { kind, .. } => format!(
                "{}_{}",
                ProviderKind::AzureDevOps.provider_prefix(),
                kind.as_ref()
            ),
            Self::Other { provider, kind, .. } => format!("{provider}_{kind}"),
            Self::Raw(value) => value
                .split_once(".")
                .map(|pair| pair.0.to_owned())
                .unwrap_or(value.to_owned()),
        }
    }
    pub fn name_label(&self) -> &str {
        match self {
            Self::AzureRM { name, .. } => name.as_str(),
            Self::AzureAD { name, .. } => name.as_str(),
            Self::AzureDevOps { name, .. } => name.as_str(),
            Self::Other { name, .. } => name.as_str(),
            Self::Raw(value) => value.split_once(".").map(|pair| pair.1).unwrap_or(value),
        }
    }
    pub fn use_name(&mut self, mapper: impl Fn(&str) -> String) -> &mut Self {
        match self {
            Self::AzureRM { name, .. } => {
                *name = (mapper)(name);
            }
            Self::AzureAD { name, .. } => {
                *name = (mapper)(name);
            }
            Self::AzureDevOps { name, .. } => {
                *name = (mapper)(name);
            }
            Self::Other { name, .. } => {
                *name = (mapper)(name);
            }
            Self::Raw(value) => {
                if let Some((kind, name)) = value.split_once('.') {
                    let new_name = (mapper)(name);
                    *value = format!("{kind}.{new_name}");
                }
            }
        };
        self
    }
}
impl std::fmt::Display for ResourceBlockReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expression_str())
    }
}
impl TryFrom<ResourceBlockReference> for Expression {
    type Error = parser::Error;

    fn try_from(value: ResourceBlockReference) -> Result<Self, Self::Error> {
        value.expression_str().parse::<Expression>()
    }
}
impl FromStr for ResourceBlockReference {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expr = s.parse::<Expression>()?;
        let rtn: ResourceBlockReference = expr.try_into()?;
        Ok(rtn)
    }
}
impl TryFrom<Expression> for ResourceBlockReference {
    type Error = eyre::Error;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        let Expression::Traversal(traversal) = expr else {
            bail!("Failed to match expression as a traversal: {expr:?}");
        };
        traversal.try_into()
    }
}
impl TryFrom<Box<Traversal>> for ResourceBlockReference {
    type Error = eyre::Error;

    fn try_from(mut traversal: Box<Traversal>) -> Result<Self, Self::Error> {
        let Expression::Variable(first) = &traversal.expr else {
            bail!("Expected traversal expr to be a variable but it wasn't: {traversal:?}");
        };
        let first = first.as_str();
        if first == "module" {
            let content = Expression::Traversal(traversal.to_owned()).to_string();
            return Ok(ResourceBlockReference::Raw(content));
        }

        let kind = ResourceBlockResourceKind::from_str(first).context(format!(
            "tried to parse {:?} from {:?} as a {:?}",
            first,
            traversal,
            type_name::<ResourceBlockResourceKind>()
        ))?;

        if traversal.operators.len() != 1 {
            bail!(
                "Expected only one traversal operator, found {} in {:?}",
                traversal.operators.len(),
                traversal
            );
        }
        let name = traversal.operators.pop().unwrap();
        let name = name.into_value();
        let TraversalOperator::GetAttr(name) = name else {
            bail!("Expected traversal operator to be GetAttr, was {name:?} from {traversal:?}");
        };
        let name = name.as_str().to_owned();

        Ok(match kind {
            ResourceBlockResourceKind::AzureAD(kind) => {
                ResourceBlockReference::AzureAD { kind, name }
            }
            ResourceBlockResourceKind::AzureRM(kind) => {
                ResourceBlockReference::AzureRM { kind, name }
            }
            ResourceBlockResourceKind::AzureDevOps(kind) => {
                ResourceBlockReference::AzureDevOps { kind, name }
            }
            ResourceBlockResourceKind::Other(kind) => ResourceBlockReference::Other {
                provider: kind.provider,
                kind: kind.resource,
                name,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() -> eyre::Result<()> {
        let x = ResourceBlockReference::AzureRM {
            kind: AzureRmResourceBlockKind::ResourceGroup,
            name: "my-rg".to_string(),
        };
        let y: Expression = x.try_into()?;
        println!("{y:?}");
        Ok(())
    }
}
