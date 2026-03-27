use crate::Principal;
use crate::PrincipalId;
use std::str::FromStr;

/// Principal can be specified as an id (UUID) or a display/user principal name.
#[derive(Debug, Clone)]
pub enum AzurePrincipalArgument<'a> {
    Id(PrincipalId),
    IdRef(&'a PrincipalId),
    Name(String),
    NameRef(&'a str),
    Principal(Principal),
    PrincipalRef(&'a Principal),
}

impl std::fmt::Display for AzurePrincipalArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzurePrincipalArgument::Id(id) => id.fmt(f),
            AzurePrincipalArgument::IdRef(id) => id.fmt(f),
            AzurePrincipalArgument::Name(s) => s.fmt(f),
            AzurePrincipalArgument::NameRef(s) => s.fmt(f),
            AzurePrincipalArgument::Principal(p) => p.fmt(f),
            AzurePrincipalArgument::PrincipalRef(p) => p.fmt(f),
        }
    }
}

impl From<PrincipalId> for AzurePrincipalArgument<'_> {
    fn from(value: PrincipalId) -> Self {
        AzurePrincipalArgument::Id(value)
    }
}
impl<'a> From<&'a PrincipalId> for AzurePrincipalArgument<'a> {
    fn from(value: &'a PrincipalId) -> Self {
        AzurePrincipalArgument::IdRef(value)
    }
}
impl From<Principal> for AzurePrincipalArgument<'_> {
    fn from(value: Principal) -> Self {
        AzurePrincipalArgument::Principal(value)
    }
}
impl<'a> From<&'a Principal> for AzurePrincipalArgument<'a> {
    fn from(value: &'a Principal) -> Self {
        AzurePrincipalArgument::PrincipalRef(value)
    }
}
impl From<String> for AzurePrincipalArgument<'_> {
    fn from(value: String) -> Self {
        AzurePrincipalArgument::Name(value)
    }
}
impl<'a> From<&'a str> for AzurePrincipalArgument<'a> {
    fn from(value: &'a str) -> Self {
        AzurePrincipalArgument::NameRef(value)
    }
}

impl AzurePrincipalArgument<'_> {
    pub fn into_owned(self) -> AzurePrincipalArgument<'static> {
        match self {
            AzurePrincipalArgument::Id(id) => AzurePrincipalArgument::Id(id),
            AzurePrincipalArgument::IdRef(id) => AzurePrincipalArgument::Id(*id),
            AzurePrincipalArgument::Name(name) => AzurePrincipalArgument::Name(name),
            AzurePrincipalArgument::NameRef(name) => AzurePrincipalArgument::Name(name.to_string()),
            AzurePrincipalArgument::Principal(p) => AzurePrincipalArgument::Principal(p),
            AzurePrincipalArgument::PrincipalRef(p) => AzurePrincipalArgument::Principal(p.clone()),
        }
    }

    pub fn matches(&self, principal: &Principal) -> bool {
        match self {
            AzurePrincipalArgument::Id(id) => principal.id() == id,
            AzurePrincipalArgument::IdRef(id) => principal.id() == *id,
            AzurePrincipalArgument::Name(name) => {
                principal.display_name().eq_ignore_ascii_case(name.as_str())
                    || principal.name().eq_ignore_ascii_case(name.as_str())
            }
            AzurePrincipalArgument::NameRef(name) => {
                principal.display_name().eq_ignore_ascii_case(name)
                    || principal.name().eq_ignore_ascii_case(name)
            }
            AzurePrincipalArgument::Principal(p) => p.as_ref() == principal.as_ref(),
            AzurePrincipalArgument::PrincipalRef(p) => p.as_ref() == principal.as_ref(),
        }
    }
}

impl FromStr for AzurePrincipalArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<PrincipalId>() {
            Ok(AzurePrincipalArgument::Id(id))
        } else {
            // treat as name / userPrincipalName
            Ok(AzurePrincipalArgument::Name(s.to_string()))
        }
    }
}
