use crate::GiteaOrganization;
use crate::GiteaOrganizationId;
use crate::GiteaOrganizationName;
use arbitrary::Arbitrary;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Arbitrary, Facet)]
#[repr(C)]
pub enum GiteaOrganizationArgument<'a> {
    Id(Cow<'a, GiteaOrganizationId>),
    Name(Cow<'a, GiteaOrganizationName>),
}

impl Display for GiteaOrganizationArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Name(name) => name.fmt(f),
        }
    }
}

impl From<GiteaOrganizationId> for GiteaOrganizationArgument<'_> {
    fn from(value: GiteaOrganizationId) -> Self {
        Self::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaOrganizationId> for GiteaOrganizationArgument<'a> {
    fn from(value: &'a GiteaOrganizationId) -> Self {
        Self::Id(Cow::Borrowed(value))
    }
}

impl From<GiteaOrganizationName> for GiteaOrganizationArgument<'_> {
    fn from(value: GiteaOrganizationName) -> Self {
        Self::Name(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaOrganizationName> for GiteaOrganizationArgument<'a> {
    fn from(value: &'a GiteaOrganizationName) -> Self {
        Self::Name(Cow::Borrowed(value))
    }
}

impl GiteaOrganizationArgument<'_> {
    pub fn into_owned(self) -> GiteaOrganizationArgument<'static> {
        match self {
            Self::Id(id) => GiteaOrganizationArgument::Id(Cow::Owned(id.into_owned())),
            Self::Name(name) => GiteaOrganizationArgument::Name(Cow::Owned(name.into_owned())),
        }
    }

    pub fn matches(&self, organization: &GiteaOrganization) -> bool {
        match self {
            Self::Id(id) => &organization.id == id.as_ref(),
            Self::Name(name) => organization
                .username
                .as_ref()
                .eq_ignore_ascii_case(name.as_ref().as_ref()),
        }
    }
}

impl FromStr for GiteaOrganizationArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<GiteaOrganizationId>() {
            Ok(Self::Id(Cow::Owned(id)))
        } else if let Ok(name) = s.parse::<GiteaOrganizationName>() {
            Ok(Self::Name(Cow::Owned(name)))
        } else {
            bail!("'{s}' is not a valid Gitea organization id or name")
        }
    }
}

impl TryFrom<String> for GiteaOrganizationArgument<'static> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&GiteaOrganizationArgument<'_>> for String {
    fn from(value: &GiteaOrganizationArgument<'_>) -> Self {
        value.to_string()
    }
}
