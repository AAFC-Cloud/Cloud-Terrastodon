use crate::RoleDefinition;
use crate::RoleDefinitionId;
use crate::RoleDefinitionName;
use crate::scopes::Scope;
use crate::slug::Slug;
use std::str::FromStr;

/// Role definition can be specified as a RoleDefinitionId (expanded form or scoped),
/// or by its name (display name / GUID form).
#[derive(Debug, Clone)]
pub enum AzureRoleDefinitionArgument<'a> {
    Id(RoleDefinitionId),
    IdRef(&'a RoleDefinitionId),
    Name(RoleDefinitionName),
    NameRef(&'a RoleDefinitionName),
    RawName(String),
    RawNameRef(&'a str),
}

impl std::fmt::Display for AzureRoleDefinitionArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureRoleDefinitionArgument::Id(id) => write!(f, "{:?}", id),
            AzureRoleDefinitionArgument::IdRef(id) => write!(f, "{:?}", id),
            AzureRoleDefinitionArgument::Name(n) => n.fmt(f),
            AzureRoleDefinitionArgument::NameRef(n) => n.fmt(f),
            AzureRoleDefinitionArgument::RawName(s) => s.fmt(f),
            AzureRoleDefinitionArgument::RawNameRef(s) => s.fmt(f),
        }
    }
}

impl From<RoleDefinitionId> for AzureRoleDefinitionArgument<'_> {
    fn from(value: RoleDefinitionId) -> Self {
        AzureRoleDefinitionArgument::Id(value)
    }
}
impl<'a> From<&'a RoleDefinitionId> for AzureRoleDefinitionArgument<'a> {
    fn from(value: &'a RoleDefinitionId) -> Self {
        AzureRoleDefinitionArgument::IdRef(value)
    }
}
impl From<RoleDefinitionName> for AzureRoleDefinitionArgument<'_> {
    fn from(value: RoleDefinitionName) -> Self {
        AzureRoleDefinitionArgument::Name(value)
    }
}
impl<'a> From<&'a RoleDefinitionName> for AzureRoleDefinitionArgument<'a> {
    fn from(value: &'a RoleDefinitionName) -> Self {
        AzureRoleDefinitionArgument::NameRef(value)
    }
}
impl From<String> for AzureRoleDefinitionArgument<'_> {
    fn from(value: String) -> Self {
        AzureRoleDefinitionArgument::RawName(value)
    }
}
impl<'a> From<&'a str> for AzureRoleDefinitionArgument<'a> {
    fn from(value: &'a str) -> Self {
        AzureRoleDefinitionArgument::RawNameRef(value)
    }
}

impl AzureRoleDefinitionArgument<'_> {
    pub fn into_owned(self) -> AzureRoleDefinitionArgument<'static> {
        match self {
            AzureRoleDefinitionArgument::Id(id) => AzureRoleDefinitionArgument::Id(id),
            AzureRoleDefinitionArgument::IdRef(id) => AzureRoleDefinitionArgument::Id(id.clone()),
            AzureRoleDefinitionArgument::Name(n) => AzureRoleDefinitionArgument::Name(n),
            AzureRoleDefinitionArgument::NameRef(n) => AzureRoleDefinitionArgument::Name(n.clone()),
            AzureRoleDefinitionArgument::RawName(s) => AzureRoleDefinitionArgument::RawName(s),
            AzureRoleDefinitionArgument::RawNameRef(s) => {
                AzureRoleDefinitionArgument::RawName(s.to_string())
            }
        }
    }

    pub fn matches(&self, role: &RoleDefinition) -> bool {
        match self {
            AzureRoleDefinitionArgument::Id(id) => &role.id == id,
            AzureRoleDefinitionArgument::IdRef(id) => &role.id == *id,
            AzureRoleDefinitionArgument::Name(name) => {
                role.display_name.eq_ignore_ascii_case(&name.to_string())
            }
            AzureRoleDefinitionArgument::NameRef(name) => {
                role.display_name.eq_ignore_ascii_case(&name.to_string())
            }
            AzureRoleDefinitionArgument::RawName(s) => {
                role.display_name.eq_ignore_ascii_case(s)
                    || role.id.expanded_form().eq_ignore_ascii_case(s)
                    || role
                        .id
                        .expanded_form()
                        .rsplit_once('/')
                        .map(|x| x.1)
                        .unwrap_or("")
                        .eq_ignore_ascii_case(s)
            }
            AzureRoleDefinitionArgument::RawNameRef(s) => {
                role.display_name.eq_ignore_ascii_case(s)
                    || role.id.expanded_form().eq_ignore_ascii_case(s)
                    || role
                        .id
                        .expanded_form()
                        .rsplit_once('/')
                        .map(|x| x.1)
                        .unwrap_or("")
                        .eq_ignore_ascii_case(s)
            }
        }
    }
}

impl FromStr for AzureRoleDefinitionArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<RoleDefinitionId>() {
            Ok(AzureRoleDefinitionArgument::Id(id))
        } else if let Ok(name) = RoleDefinitionName::try_new(s.to_string()) {
            Ok(AzureRoleDefinitionArgument::Name(name))
        } else {
            // fallback to raw name
            Ok(AzureRoleDefinitionArgument::RawName(s.to_string()))
        }
    }
}
