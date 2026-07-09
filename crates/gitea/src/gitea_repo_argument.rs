use crate::GiteaRepo;
use crate::GiteaRepoFullName;
use crate::GiteaRepoId;
use crate::GiteaRepoName;
use arbitrary::Arbitrary;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Arbitrary, Facet)]
#[repr(C)]
pub enum GiteaRepoArgument<'a> {
    Id(Cow<'a, GiteaRepoId>),
    FullName(Cow<'a, GiteaRepoFullName>),
    Name(Cow<'a, GiteaRepoName>),
}

impl Display for GiteaRepoArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::FullName(full_name) => full_name.fmt(f),
            Self::Name(name) => name.fmt(f),
        }
    }
}

impl From<GiteaRepoId> for GiteaRepoArgument<'_> {
    fn from(value: GiteaRepoId) -> Self {
        Self::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaRepoId> for GiteaRepoArgument<'a> {
    fn from(value: &'a GiteaRepoId) -> Self {
        Self::Id(Cow::Borrowed(value))
    }
}

impl From<GiteaRepoFullName> for GiteaRepoArgument<'_> {
    fn from(value: GiteaRepoFullName) -> Self {
        Self::FullName(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaRepoFullName> for GiteaRepoArgument<'a> {
    fn from(value: &'a GiteaRepoFullName) -> Self {
        Self::FullName(Cow::Borrowed(value))
    }
}

impl From<GiteaRepoName> for GiteaRepoArgument<'_> {
    fn from(value: GiteaRepoName) -> Self {
        Self::Name(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaRepoName> for GiteaRepoArgument<'a> {
    fn from(value: &'a GiteaRepoName) -> Self {
        Self::Name(Cow::Borrowed(value))
    }
}

impl GiteaRepoArgument<'_> {
    pub fn into_owned(self) -> GiteaRepoArgument<'static> {
        match self {
            Self::Id(id) => GiteaRepoArgument::Id(Cow::Owned(id.into_owned())),
            Self::FullName(full_name) => {
                GiteaRepoArgument::FullName(Cow::Owned(full_name.into_owned()))
            }
            Self::Name(name) => GiteaRepoArgument::Name(Cow::Owned(name.into_owned())),
        }
    }

    pub fn matches(&self, repo: &GiteaRepo) -> bool {
        match self {
            Self::Id(id) => &repo.id == id.as_ref(),
            Self::FullName(full_name) => repo
                .full_name
                .to_string()
                .eq_ignore_ascii_case(&full_name.as_ref().to_string()),
            Self::Name(name) => repo
                .name
                .as_ref()
                .eq_ignore_ascii_case(name.as_ref().as_ref()),
        }
    }
}

impl FromStr for GiteaRepoArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<GiteaRepoId>() {
            Ok(Self::Id(Cow::Owned(id)))
        } else if let Ok(full_name) = s.parse::<GiteaRepoFullName>() {
            Ok(Self::FullName(Cow::Owned(full_name)))
        } else if let Ok(name) = s.parse::<GiteaRepoName>() {
            Ok(Self::Name(Cow::Owned(name)))
        } else {
            bail!("'{s}' is not a valid Gitea repository id, full name, or name")
        }
    }
}

impl TryFrom<String> for GiteaRepoArgument<'static> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&GiteaRepoArgument<'_>> for String {
    fn from(value: &GiteaRepoArgument<'_>) -> Self {
        value.to_string()
    }
}
