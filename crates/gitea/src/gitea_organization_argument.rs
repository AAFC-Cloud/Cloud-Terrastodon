use crate::GiteaOrganization;
use crate::GiteaOrganizationId;
use crate::GiteaOrganizationName;
use eyre::bail;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum GiteaOrganizationArgument<'a> {
    Id(GiteaOrganizationId),
    IdRef(&'a GiteaOrganizationId),
    Name(GiteaOrganizationName),
    NameRef(&'a GiteaOrganizationName),
}

impl Display for GiteaOrganizationArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::IdRef(id) => id.fmt(f),
            Self::Name(name) => name.fmt(f),
            Self::NameRef(name) => name.fmt(f),
        }
    }
}

impl GiteaOrganizationArgument<'_> {
    pub fn matches(&self, organization: &GiteaOrganization) -> bool {
        match self {
            Self::Id(id) => organization.id == *id,
            Self::IdRef(id) => organization.id == **id,
            Self::Name(name) => organization
                .username
                .as_ref()
                .eq_ignore_ascii_case(name.as_ref()),
            Self::NameRef(name) => organization
                .username
                .as_ref()
                .eq_ignore_ascii_case(name.as_ref()),
        }
    }
}

impl FromStr for GiteaOrganizationArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<GiteaOrganizationId>() {
            Ok(Self::Id(id))
        } else if let Ok(name) = s.parse::<GiteaOrganizationName>() {
            Ok(Self::Name(name))
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
