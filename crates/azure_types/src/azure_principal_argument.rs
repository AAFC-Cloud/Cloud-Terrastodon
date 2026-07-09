use crate::Principal;
use crate::PrincipalCollection;
use crate::PrincipalId;
use crate::PrincipalRef;
use arbitrary::Arbitrary;
use std::borrow::Cow;
use std::str::FromStr;

/// Principal can be specified as an id (UUID) or a display/user principal name.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzurePrincipalArgument<'a> {
    Id(Cow<'a, PrincipalId>),
    Name(Cow<'a, str>),
    Principal(Cow<'a, Principal>),
}

impl std::fmt::Display for AzurePrincipalArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzurePrincipalArgument::Id(id) => id.fmt(f),
            AzurePrincipalArgument::Name(s) => s.fmt(f),
            AzurePrincipalArgument::Principal(p) => p.fmt(f),
        }
    }
}

impl From<PrincipalId> for AzurePrincipalArgument<'_> {
    fn from(value: PrincipalId) -> Self {
        AzurePrincipalArgument::Id(Cow::Owned(value))
    }
}
impl<'a> From<&'a PrincipalId> for AzurePrincipalArgument<'a> {
    fn from(value: &'a PrincipalId) -> Self {
        AzurePrincipalArgument::Id(Cow::Borrowed(value))
    }
}
impl From<Principal> for AzurePrincipalArgument<'_> {
    fn from(value: Principal) -> Self {
        AzurePrincipalArgument::Principal(Cow::Owned(value))
    }
}
impl<'a> From<&'a Principal> for AzurePrincipalArgument<'a> {
    fn from(value: &'a Principal) -> Self {
        AzurePrincipalArgument::Principal(Cow::Borrowed(value))
    }
}
impl<'a> From<&'a str> for AzurePrincipalArgument<'a> {
    fn from(value: &'a str) -> Self {
        AzurePrincipalArgument::Name(Cow::Borrowed(value))
    }
}

impl AzurePrincipalArgument<'_> {
    pub fn as_id(&self) -> Option<&PrincipalId> {
        match self {
            AzurePrincipalArgument::Id(id) => Some(id.as_ref()),
            AzurePrincipalArgument::Name(..) | AzurePrincipalArgument::Principal(..) => None,
        }
    }

    pub fn into_owned(self) -> AzurePrincipalArgument<'static> {
        match self {
            AzurePrincipalArgument::Id(id) => {
                AzurePrincipalArgument::Id(Cow::Owned(id.into_owned()))
            }
            AzurePrincipalArgument::Name(name) => {
                AzurePrincipalArgument::Name(Cow::Owned(name.into_owned()))
            }
            AzurePrincipalArgument::Principal(p) => {
                AzurePrincipalArgument::Principal(Cow::Owned(p.into_owned()))
            }
        }
    }

    pub fn resolve<'a>(&self, principals: &'a PrincipalCollection) -> Option<&'a Principal> {
        principals
            .values()
            .find(|principal| self.matches(*principal))
    }

    pub fn matches<'a>(&self, principal: impl Into<PrincipalRef<'a>>) -> bool {
        let principal = principal.into();
        match self {
            AzurePrincipalArgument::Id(id) => principal.id() == *id.as_ref(),
            AzurePrincipalArgument::Name(name) => {
                principal.display_name().eq_ignore_ascii_case(name.as_ref())
                    || principal.name().eq_ignore_ascii_case(name.as_ref())
            }
            AzurePrincipalArgument::Principal(p) => p.id() == principal.id(),
        }
    }
}

impl<'a> FromStr for AzurePrincipalArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<PrincipalId>() {
            Ok(AzurePrincipalArgument::Id(Cow::Owned(id)))
        } else {
            // treat as name / userPrincipalName
            Ok(AzurePrincipalArgument::Name(Cow::Owned(s.to_string())))
        }
    }
}

impl<'a> TryFrom<String> for AzurePrincipalArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzurePrincipalArgument<'_>> for String {
    fn from(value: &AzurePrincipalArgument<'_>) -> Self {
        value.to_string()
    }
}
