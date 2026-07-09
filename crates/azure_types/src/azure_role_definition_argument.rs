use crate::RoleDefinition;
use crate::RoleDefinitionId;
use crate::RoleDefinitionName;
use crate::scopes::Scope;
use crate::slug::Slug;
use arbitrary::Arbitrary;
use std::borrow::Cow;
use std::str::FromStr;

/// Role definition can be specified as a RoleDefinitionId (expanded form or scoped),
/// or by its name (display name / GUID form).
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureRoleDefinitionArgument<'a> {
    Id(Cow<'a, RoleDefinitionId>),
    Name(Cow<'a, RoleDefinitionName>),
    RawName(Cow<'a, str>),
}

impl std::fmt::Display for AzureRoleDefinitionArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureRoleDefinitionArgument::Id(id) => write!(f, "{:?}", id.as_ref()),
            AzureRoleDefinitionArgument::Name(n) => n.fmt(f),
            AzureRoleDefinitionArgument::RawName(s) => s.fmt(f),
        }
    }
}

impl From<RoleDefinitionId> for AzureRoleDefinitionArgument<'_> {
    fn from(value: RoleDefinitionId) -> Self {
        AzureRoleDefinitionArgument::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a RoleDefinitionId> for AzureRoleDefinitionArgument<'a> {
    fn from(value: &'a RoleDefinitionId) -> Self {
        AzureRoleDefinitionArgument::Id(Cow::Borrowed(value))
    }
}

impl From<RoleDefinitionName> for AzureRoleDefinitionArgument<'_> {
    fn from(value: RoleDefinitionName) -> Self {
        AzureRoleDefinitionArgument::Name(Cow::Owned(value))
    }
}

impl<'a> From<&'a RoleDefinitionName> for AzureRoleDefinitionArgument<'a> {
    fn from(value: &'a RoleDefinitionName) -> Self {
        AzureRoleDefinitionArgument::Name(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a str> for AzureRoleDefinitionArgument<'a> {
    fn from(value: &'a str) -> Self {
        AzureRoleDefinitionArgument::RawName(Cow::Borrowed(value))
    }
}

impl AzureRoleDefinitionArgument<'_> {
    pub fn into_owned(self) -> AzureRoleDefinitionArgument<'static> {
        match self {
            AzureRoleDefinitionArgument::Id(id) => {
                AzureRoleDefinitionArgument::Id(Cow::Owned(id.into_owned()))
            }
            AzureRoleDefinitionArgument::Name(n) => {
                AzureRoleDefinitionArgument::Name(Cow::Owned(n.into_owned()))
            }
            AzureRoleDefinitionArgument::RawName(s) => {
                AzureRoleDefinitionArgument::RawName(Cow::Owned(s.into_owned()))
            }
        }
    }

    pub fn matches(&self, role: &RoleDefinition) -> bool {
        match self {
            AzureRoleDefinitionArgument::Id(id) => &role.id == id.as_ref(),
            AzureRoleDefinitionArgument::Name(name) => role
                .display_name
                .eq_ignore_ascii_case(&name.as_ref().to_string()),
            AzureRoleDefinitionArgument::RawName(s) => {
                role.display_name.eq_ignore_ascii_case(s.as_ref())
                    || role.id.expanded_form().eq_ignore_ascii_case(s.as_ref())
                    || role
                        .id
                        .expanded_form()
                        .rsplit_once('/')
                        .map(|x| x.1)
                        .unwrap_or("")
                        .eq_ignore_ascii_case(s.as_ref())
            }
        }
    }
}

impl<'a> FromStr for AzureRoleDefinitionArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<RoleDefinitionId>() {
            Ok(AzureRoleDefinitionArgument::Id(Cow::Owned(id)))
        } else if let Ok(name) = RoleDefinitionName::try_new(s.to_string()) {
            Ok(AzureRoleDefinitionArgument::Name(Cow::Owned(name)))
        } else {
            Ok(AzureRoleDefinitionArgument::RawName(Cow::Owned(
                s.to_string(),
            )))
        }
    }
}

impl<'a> TryFrom<String> for AzureRoleDefinitionArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureRoleDefinitionArgument<'_>> for String {
    fn from(value: &AzureRoleDefinitionArgument<'_>) -> Self {
        value.to_string()
    }
}
