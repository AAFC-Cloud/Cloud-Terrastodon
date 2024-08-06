use crate::providers::TofuProviderKind;
use anyhow::bail;
use hcl::edit::expr::Expression;
use hcl::edit::parser;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TofuResourceKind {
    AzureAD(TofuAzureADResourceKind),
    AzureRM(TofuAzureRMResourceKind),
    Other(String),
}
impl std::fmt::Display for TofuResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TofuResourceKind::AzureAD(res) => res.fmt(f),
            TofuResourceKind::AzureRM(res) => res.fmt(f),
            TofuResourceKind::Other(res) => f.write_str(res),
        }
    }
}
impl FromStr for TofuResourceKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(azurerm) = s.parse::<TofuAzureRMResourceKind>() {
            return Ok(TofuResourceKind::AzureRM(azurerm));
        }
        if let Ok(azuread) = s.parse::<TofuAzureADResourceKind>() {
            return Ok(TofuResourceKind::AzureAD(azuread));
        }
        Ok(TofuResourceKind::Other(s.to_owned()))
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = TofuProviderKind::AzureRM.provider_prefix();
        let Some(seeking) = s.strip_prefix(provider_prefix).and_then(|s| s.strip_prefix("_")) else {
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = TofuProviderKind::AzureAD.provider_prefix();
        let Some(seeking) = s.strip_prefix(provider_prefix).and_then(|s| s.strip_prefix("_")) else {
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
        provider_kind: TofuProviderKind,
        kind: String,
        name: String,
    },
    Raw(String),
}
impl TofuResourceReference {
    pub fn expression_str(&self) -> String {
        format!("{}.{}", self.kind_label(), self.name_label())
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
            Self::Other {
                provider_kind: provider,
                kind,
                ..
            } => format!("{}_{}", provider, kind),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() -> anyhow::Result<()> {
        let x = TofuResourceReference::AzureRM {
            kind: TofuAzureRMResourceKind::ResourceGroup,
            name: "my-rg".to_string(),
        };
        let y: Expression = x.try_into()?;
        println!("{y:?}");
        Ok(())
    }

    #[test]
    fn parse_azurerm_role_assignment() -> anyhow::Result<()> {
        let kind: TofuResourceKind = "azurerm_role_assignment".parse()?;
        assert_eq!(kind, TofuResourceKind::AzureRM(TofuAzureRMResourceKind::RoleAssignment));
        Ok(())
    }

    #[test]
    fn parse_azurerm_other() -> anyhow::Result<()> {
        let kind: TofuResourceKind = "azurerm_synapse_workspace".parse()?;
        assert_eq!(kind, TofuResourceKind::AzureRM(TofuAzureRMResourceKind::Other("synapse_workspace".to_owned())));
        Ok(())
    }
    
    #[test]
    fn parse_azuread_group() -> anyhow::Result<()> {
        let kind: TofuResourceKind = "azuread_group".parse()?;
        assert_eq!(kind, TofuResourceKind::AzureAD(TofuAzureADResourceKind::Group));
        Ok(())
    }
    #[test]
    fn parse_azuread_other() -> anyhow::Result<()> {
        let kind: TofuResourceKind = "azuread_thingy".parse()?;
        assert_eq!(kind, TofuResourceKind::AzureAD(TofuAzureADResourceKind::Other("thingy".to_owned())));
        Ok(())
    }
}
