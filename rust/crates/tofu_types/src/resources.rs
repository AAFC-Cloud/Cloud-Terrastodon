use std::str::FromStr;

use anyhow::anyhow;

use crate::providers::TofuProviderKind;

#[derive(Debug, Clone)]
pub enum TofuAzureRMResourceKind {
    ManagementGroupPolicyAssignment,
    ResourceGroup,
    PolicyAssignment,
    PolicyDefinition,
    PolicySetDefinition,
    RoleAssignment,
    Other(String),
}
impl TofuAzureRMResourceKind {
    pub fn supported_variants() -> Vec<TofuAzureRMResourceKind> {
        vec![
            TofuAzureRMResourceKind::ManagementGroupPolicyAssignment,
            TofuAzureRMResourceKind::ResourceGroup,
            TofuAzureRMResourceKind::PolicyAssignment,
            TofuAzureRMResourceKind::PolicyDefinition,
            TofuAzureRMResourceKind::PolicySetDefinition,
            TofuAzureRMResourceKind::RoleAssignment,
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
            Self::Other(s) => s.as_ref(),
        }
    }
}
impl FromStr for TofuAzureRMResourceKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seeking = s.trim_start_matches(TofuProviderKind::AzureRM.provider_prefix());
        Self::supported_variants()
            .into_iter()
            .find(|x| x.as_ref() == seeking)
            .ok_or(anyhow!("no variant matches"))
    }
}

#[derive(Debug, Clone)]
pub enum TofuAzureADResourceKind {
    Group,
    User,
    Other(String),
}
impl TofuAzureADResourceKind {
    pub fn supported_variants() -> Vec<TofuAzureADResourceKind> {
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
        let seeking = s.trim_start_matches(TofuProviderKind::AzureAD.provider_prefix());
        Self::supported_variants()
            .into_iter()
            .find(|x| x.as_ref() == seeking)
            .ok_or(anyhow!("no variant matches"))
    }
}

#[derive(Debug, Clone)]
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
    pub fn expression(&self) -> String {
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
            Self::Other { provider_kind: provider, kind, .. } => format!("{}_{}", provider, kind),
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
        f.write_str(&self.expression())
    }
}
