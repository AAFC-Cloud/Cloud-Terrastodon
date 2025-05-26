use crate::providers::ProviderKind;
use eyre::Context;
use eyre::bail;
use hcl::edit::expr::Expression;
use hcl::edit::expr::Traversal;
use hcl::edit::expr::TraversalOperator;
use hcl::edit::parser;
use std::any::type_name;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResourceBlockKind {
    AzureAD(AzureADResourceBlockKind),
    AzureRM(AzureRMResourceBlockKind),
    AzureDevOps(AzureDevOpsResourceBlockKind),
    Other(OtherResourceBlockKind),
}
impl std::fmt::Display for ResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceBlockKind::AzureAD(res) => res.fmt(f),
            ResourceBlockKind::AzureRM(res) => res.fmt(f),
            ResourceBlockKind::AzureDevOps(res) => res.fmt(f),
            ResourceBlockKind::Other(res) => res.fmt(f),
        }
    }
}
impl FromStr for ResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(kind) = s.parse::<AzureRMResourceBlockKind>() {
            return Ok(ResourceBlockKind::AzureRM(kind));
        }
        if let Ok(kind) = s.parse::<AzureADResourceBlockKind>() {
            return Ok(ResourceBlockKind::AzureAD(kind));
        }
        if let Ok(kind) = s.parse::<AzureDevOpsResourceBlockKind>() {
            return Ok(ResourceBlockKind::AzureDevOps(kind));
        }
        Ok(ResourceBlockKind::Other(OtherResourceBlockKind::from_str(
            s,
        )?))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OtherResourceBlockKind {
    provider: String,
    resource: String,
}
impl FromStr for OtherResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((provider, resource)) = s.split_once('_') else {
            bail!(
                "Expected at least one underscore in {s:?} to parse as a {:?}",
                type_name::<OtherResourceBlockKind>()
            )
        };
        Ok(OtherResourceBlockKind {
            provider: provider.to_owned(),
            resource: resource.to_owned(),
        })
    }
}
impl std::fmt::Display for OtherResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.provider)?;
        f.write_str("_")?;
        f.write_str(&self.resource)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureRMResourceBlockKind {
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
impl AzureRMResourceBlockKind {
    pub fn known_variants() -> Vec<AzureRMResourceBlockKind> {
        vec![
            AzureRMResourceBlockKind::ManagementGroupPolicyAssignment,
            AzureRMResourceBlockKind::ResourceGroup,
            AzureRMResourceBlockKind::PolicyAssignment,
            AzureRMResourceBlockKind::PolicyDefinition,
            AzureRMResourceBlockKind::PolicySetDefinition,
            AzureRMResourceBlockKind::RoleAssignment,
            AzureRMResourceBlockKind::RoleDefinition,
            AzureRMResourceBlockKind::StorageAccount,
        ]
    }
}
impl AsRef<str> for AzureRMResourceBlockKind {
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
impl FromStr for AzureRMResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = ProviderKind::AzureRM.provider_prefix();
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
impl std::fmt::Display for AzureRMResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureRM.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureADResourceBlockKind {
    Group,
    User,
    Other(String),
}
impl AzureADResourceBlockKind {
    pub fn known_variants() -> Vec<AzureADResourceBlockKind> {
        vec![
            AzureADResourceBlockKind::Group,
            AzureADResourceBlockKind::User,
        ]
    }
}
impl AsRef<str> for AzureADResourceBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Group => "group",
            Self::User => "user",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureADResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = ProviderKind::AzureAD.provider_prefix();
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

impl std::fmt::Display for AzureADResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureAD.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AzureDevOpsResourceBlockKind {
    Project,
    Team,
    Repo,
    Other(String),
}
impl AzureDevOpsResourceBlockKind {
    pub fn known_variants() -> Vec<AzureDevOpsResourceBlockKind> {
        vec![
            AzureDevOpsResourceBlockKind::Project,
            AzureDevOpsResourceBlockKind::Repo,
        ]
    }
}
impl AsRef<str> for AzureDevOpsResourceBlockKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Project => "project",
            Self::Repo => "git_repository",
            Self::Team => "team",
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for AzureDevOpsResourceBlockKind {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let provider_prefix = ProviderKind::AzureDevOps.provider_prefix();
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

impl std::fmt::Display for AzureDevOpsResourceBlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ProviderKind::AzureDevOps.provider_prefix())?;
        f.write_str("_")?;
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ResourceBlockReference {
    AzureRM {
        kind: AzureRMResourceBlockKind,
        name: String,
    },
    AzureAD {
        kind: AzureADResourceBlockKind,
        name: String,
    },
    AzureDevOps {
        kind: AzureDevOpsResourceBlockKind,
        name: String,
    },
    Other {
        provider: String,
        kind: String,
        name: String,
    },
    Raw(String),
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
            ResourceBlockReference::Other { provider, .. } => {
                ProviderKind::Other(provider.to_owned())
            }
            ResourceBlockReference::Raw(s) => {
                ProviderKind::Other(s.split_once('_').map(|x| x.0).unwrap_or(s).to_owned())
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

        let kind = ResourceBlockKind::from_str(first).context(format!(
            "tried to parse {:?} from {:?} as a {:?}",
            first,
            traversal,
            type_name::<ResourceBlockKind>()
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
            ResourceBlockKind::AzureAD(kind) => ResourceBlockReference::AzureAD { kind, name },
            ResourceBlockKind::AzureRM(kind) => ResourceBlockReference::AzureRM { kind, name },
            ResourceBlockKind::AzureDevOps(kind) => {
                ResourceBlockReference::AzureDevOps { kind, name }
            }
            ResourceBlockKind::Other(kind) => ResourceBlockReference::Other {
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
            kind: AzureRMResourceBlockKind::ResourceGroup,
            name: "my-rg".to_string(),
        };
        let y: Expression = x.try_into()?;
        println!("{y:?}");
        Ok(())
    }

    #[test]
    fn parse_azurerm_role_assignment() -> eyre::Result<()> {
        let kind: ResourceBlockKind = "azurerm_role_assignment".parse()?;
        assert_eq!(
            kind,
            ResourceBlockKind::AzureRM(AzureRMResourceBlockKind::RoleAssignment)
        );
        Ok(())
    }

    #[test]
    fn parse_azurerm_other() -> eyre::Result<()> {
        let kind: ResourceBlockKind = "azurerm_synapse_workspace".parse()?;
        assert_eq!(
            kind,
            ResourceBlockKind::AzureRM(AzureRMResourceBlockKind::Other(
                "synapse_workspace".to_owned()
            ))
        );
        Ok(())
    }

    #[test]
    fn parse_azuread_group() -> eyre::Result<()> {
        let kind: ResourceBlockKind = "azuread_group".parse()?;
        assert_eq!(
            kind,
            ResourceBlockKind::AzureAD(AzureADResourceBlockKind::Group)
        );
        Ok(())
    }
    #[test]
    fn parse_azuread_other() -> eyre::Result<()> {
        let kind: ResourceBlockKind = "azuread_thingy".parse()?;
        assert_eq!(
            kind,
            ResourceBlockKind::AzureAD(AzureADResourceBlockKind::Other("thingy".to_owned()))
        );
        Ok(())
    }
    #[test]
    fn parse_resource_reference() -> eyre::Result<()> {
        let thing = "azurerm_storage_account.bruh";
        let x: ResourceBlockReference = thing.parse()?;
        let expected = ResourceBlockReference::AzureRM {
            kind: AzureRMResourceBlockKind::StorageAccount,
            name: "bruh".to_owned(),
        };
        assert_eq!(x, expected);
        Ok(())
    }
    #[test]
    fn parse_azurerm_resource_kind() -> eyre::Result<()> {
        let thing = "azurerm_storage_account";
        let x: AzureRMResourceBlockKind = thing.parse()?;
        let expected = AzureRMResourceBlockKind::StorageAccount;
        assert_eq!(x, expected);
        Ok(())
    }
}
