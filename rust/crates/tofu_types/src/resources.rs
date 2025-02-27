use crate::providers::TofuProviderKind;
use eyre::Context;
use eyre::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::Traversal;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use std::any::type_name;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuResourceKind {
    AzureAD(TofuAzureADResourceKind),
    AzureRM(TofuAzureRMResourceKind),
    Other(TofuOtherResourceKind),
}
impl std::fmt::Display for TofuResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TofuResourceKind::AzureAD(res) => res.fmt(f),
            TofuResourceKind::AzureRM(res) => res.fmt(f),
            TofuResourceKind::Other(res) => res.fmt(f),
        }
    }
}
impl FromStr for TofuResourceKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(azurerm) = s.parse::<TofuAzureRMResourceKind>() {
            return Ok(TofuResourceKind::AzureRM(azurerm));
        }
        if let Ok(azuread) = s.parse::<TofuAzureADResourceKind>() {
            return Ok(TofuResourceKind::AzureAD(azuread));
        }
        Ok(TofuResourceKind::Other(TofuOtherResourceKind::from_str(s)?))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TofuOtherResourceKind {
    provider: String,
    resource: String,
}
impl FromStr for TofuOtherResourceKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((provider, resource)) = s.split_once('_') else {
            bail!(
                "Expected at least one underscore in {s:?} to parse as a {:?}",
                type_name::<TofuOtherResourceKind>()
            )
        };
        Ok(TofuOtherResourceKind {
            provider: provider.to_owned(),
            resource: resource.to_owned(),
        })
    }
}
impl std::fmt::Display for TofuOtherResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.provider)?;
        f.write_str("_")?;
        f.write_str(&self.resource)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuAzureRMResourceKind {
    ManagementGroupPolicyAssignment,
    ResourceGroup,
    PolicyAssignment,
    PolicyDefinition,
    PolicySetDefinition,
    RoleAssignment,
    RoleDefinition,
    StorageAccount,
    Other(String),
}
impl TofuAzureRMResourceKind {
    pub fn known_variants() -> Vec<TofuAzureRMResourceKind> {
        vec![
            TofuAzureRMResourceKind::ManagementGroupPolicyAssignment,
            TofuAzureRMResourceKind::ResourceGroup,
            TofuAzureRMResourceKind::PolicyAssignment,
            TofuAzureRMResourceKind::PolicyDefinition,
            TofuAzureRMResourceKind::PolicySetDefinition,
            TofuAzureRMResourceKind::RoleAssignment,
            TofuAzureRMResourceKind::RoleDefinition,
            TofuAzureRMResourceKind::StorageAccount,
        ]
    }
}
impl AsRef<str> for TofuAzureRMResourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::ManagementGroupPolicyAssignment => "management_group_policy_assignment",
            Self::PolicyAssignment => "policy_assignment",
            Self::ResourceGroup => "resource_group",
            Self::PolicyDefinition => "policy_definition",
            Self::PolicySetDefinition => "policy_set_definition",
            Self::RoleAssignment => "role_assignment",
            Self::RoleDefinition => "role_definition",
            Self::StorageAccount => "storage_account",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for TofuAzureRMResourceKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = TofuProviderKind::AzureRM.provider_prefix();
        let Some(seeking) = s
            .strip_prefix(provider_prefix)
            .and_then(|s| s.strip_prefix("_"))
        else {
            bail!(format!(
                "String {s:?} is missing prefix {}",
                provider_prefix
            ));
        };
        for variant in Self::known_variants() {
            if variant.as_ref() == seeking {
                return Ok(variant);
            }
        }
        Ok(Self::Other(seeking.to_owned()))
    }
}
impl std::fmt::Display for TofuAzureRMResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(TofuProviderKind::AzureRM.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuAzureADResourceKind {
    Group,
    User,
    Other(String),
}
impl TofuAzureADResourceKind {
    pub fn known_variants() -> Vec<TofuAzureADResourceKind> {
        vec![
            TofuAzureADResourceKind::Group,
            TofuAzureADResourceKind::User,
        ]
    }
}
impl AsRef<str> for TofuAzureADResourceKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Group => "group",
            Self::User => "user",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for TofuAzureADResourceKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = TofuProviderKind::AzureAD.provider_prefix();
        let Some(seeking) = s
            .strip_prefix(provider_prefix)
            .and_then(|s| s.strip_prefix("_"))
        else {
            bail!(format!(
                "String {s:?} is missing prefix {}",
                provider_prefix
            ));
        };
        for variant in Self::known_variants() {
            if variant.as_ref() == seeking {
                return Ok(variant);
            }
        }
        Ok(Self::Other(seeking.to_owned()))
    }
}

impl std::fmt::Display for TofuAzureADResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(TofuProviderKind::AzureAD.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TofuResourceReference {
    AzureRM {
        kind: TofuAzureRMResourceKind,
        name: String,
    },
    AzureAD {
        kind: TofuAzureADResourceKind,
        name: String,
    },
    Other {
        provider: String,
        kind: String,
        name: String,
    },
    Raw(String),
}
impl TofuResourceReference {
    pub fn expression_str(&self) -> String {
        format!("{}.{}", self.kind_label(), self.name_label())
    }
    pub fn provider_kind(&self) -> TofuProviderKind {
        match self {
            TofuResourceReference::AzureRM { .. } => TofuProviderKind::AzureRM,
            TofuResourceReference::AzureAD { .. } => TofuProviderKind::AzureAD,
            TofuResourceReference::Other { provider, .. } => {
                TofuProviderKind::Other(provider.to_owned())
            }
            TofuResourceReference::Raw(s) => {
                TofuProviderKind::Other(s.split_once('_').map(|x| x.0).unwrap_or(s).to_owned())
            }
        }
    }
    pub fn kind_label(&self) -> String {
        match self {
            Self::AzureRM { kind, .. } => format!(
                "{}_{}",
                TofuProviderKind::AzureRM.provider_prefix(),
                kind.as_ref()
            ),
            Self::AzureAD { kind, .. } => format!(
                "{}_{}",
                TofuProviderKind::AzureAD.provider_prefix(),
                kind.as_ref()
            ),
            Self::Other { provider, kind, .. } => format!("{}_{}", provider, kind),
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
            Self::Other { name, .. } => {
                *name = (mapper)(name);
            }
            Self::Raw(value) => {
                if let Some((kind, name)) = value.split_once('.') {
                    let new_name = (mapper)(name);
                    *value = format!("{}.{}", kind, new_name);
                }
            }
        };
        self
    }
}
impl std::fmt::Display for TofuResourceReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expression_str())
    }
}
impl TryFrom<TofuResourceReference> for Expression {
    type Error = parser::Error;

    fn try_from(value: TofuResourceReference) -> Result<Self, Self::Error> {
        value.expression_str().parse::<Expression>()
    }
}
impl FromStr for TofuResourceReference {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let expr = s.parse::<Expression>()?;
        let rtn: TofuResourceReference = expr.try_into()?;
        Ok(rtn)
    }
}
impl TryFrom<Expression> for TofuResourceReference {
    type Error = eyre::Error;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        let Expression::Traversal(traversal) = expr else {
            bail!("Failed to match expression as a traversal: {expr:?}");
        };
        traversal.try_into()
    }
}
impl TryFrom<Box<Traversal>> for TofuResourceReference {
    type Error = eyre::Error;

    fn try_from(mut traversal: Box<Traversal>) -> Result<Self, Self::Error> {
        let Expression::Variable(first) = &traversal.expr else {
            bail!("Expected traversal expr to be a variable but it wasn't: {traversal:?}");
        };
        let first = first.as_str();
        if first == "module" {
            let content = Expression::Traversal(traversal.to_owned()).to_string();
            return Ok(TofuResourceReference::Raw(content));
        }

        let kind = TofuResourceKind::from_str(first).context(format!(
            "tried to parse {:?} from {:?} as a {:?}",
            first,
            traversal,
            type_name::<TofuResourceKind>()
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
            TofuResourceKind::AzureAD(kind) => TofuResourceReference::AzureAD { kind, name },
            TofuResourceKind::AzureRM(kind) => TofuResourceReference::AzureRM { kind, name },
            TofuResourceKind::Other(kind) => TofuResourceReference::Other {
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
        let x = TofuResourceReference::AzureRM {
            kind: TofuAzureRMResourceKind::ResourceGroup,
            name: "my-rg".to_string(),
        };
        let y: Expression = x.try_into()?;
        println!("{y:?}");
        Ok(())
    }

    #[test]
    fn parse_azurerm_role_assignment() -> eyre::Result<()> {
        let kind: TofuResourceKind = "azurerm_role_assignment".parse()?;
        assert_eq!(
            kind,
            TofuResourceKind::AzureRM(TofuAzureRMResourceKind::RoleAssignment)
        );
        Ok(())
    }

    #[test]
    fn parse_azurerm_other() -> eyre::Result<()> {
        let kind: TofuResourceKind = "azurerm_synapse_workspace".parse()?;
        assert_eq!(
            kind,
            TofuResourceKind::AzureRM(TofuAzureRMResourceKind::Other(
                "synapse_workspace".to_owned()
            ))
        );
        Ok(())
    }

    #[test]
    fn parse_azuread_group() -> eyre::Result<()> {
        let kind: TofuResourceKind = "azuread_group".parse()?;
        assert_eq!(
            kind,
            TofuResourceKind::AzureAD(TofuAzureADResourceKind::Group)
        );
        Ok(())
    }
    #[test]
    fn parse_azuread_other() -> eyre::Result<()> {
        let kind: TofuResourceKind = "azuread_thingy".parse()?;
        assert_eq!(
            kind,
            TofuResourceKind::AzureAD(TofuAzureADResourceKind::Other("thingy".to_owned()))
        );
        Ok(())
    }
    #[test]
    fn parse_tofu_resource_reference1() -> eyre::Result<()> {
        let thing = "azurerm_storage_account.bruh";
        let x: TofuResourceReference = thing.parse()?;
        let expected = TofuResourceReference::AzureRM {
            kind: TofuAzureRMResourceKind::StorageAccount,
            name: "bruh".to_owned(),
        };
        assert_eq!(x, expected);
        Ok(())
    }
    #[test]
    fn parse_tofu_azurerm_resource_kind() -> eyre::Result<()> {
        let thing = "azurerm_storage_account";
        let x: TofuAzureRMResourceKind = thing.parse()?;
        let expected = TofuAzureRMResourceKind::StorageAccount;
        assert_eq!(x, expected);
        Ok(())
    }
}
