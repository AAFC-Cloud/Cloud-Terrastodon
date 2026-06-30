use crate::GiteaRepo;
use crate::GiteaRepoFullName;
use crate::GiteaRepoId;
use crate::GiteaRepoName;
use eyre::bail;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum GiteaRepoArgument<'a> {
    Id(GiteaRepoId),
    IdRef(&'a GiteaRepoId),
    FullName(GiteaRepoFullName),
    FullNameRef(&'a GiteaRepoFullName),
    Name(GiteaRepoName),
    NameRef(&'a GiteaRepoName),
}

impl Display for GiteaRepoArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::IdRef(id) => id.fmt(f),
            Self::FullName(full_name) => full_name.fmt(f),
            Self::FullNameRef(full_name) => full_name.fmt(f),
            Self::Name(name) => name.fmt(f),
            Self::NameRef(name) => name.fmt(f),
        }
    }
}

impl GiteaRepoArgument<'_> {
    pub fn matches(&self, repo: &GiteaRepo) -> bool {
        match self {
            Self::Id(id) => repo.id == *id,
            Self::IdRef(id) => repo.id == **id,
            Self::FullName(full_name) => repo
                .full_name
                .to_string()
                .eq_ignore_ascii_case(&full_name.to_string()),
            Self::FullNameRef(full_name) => repo
                .full_name
                .to_string()
                .eq_ignore_ascii_case(&full_name.to_string()),
            Self::Name(name) => repo.name.as_ref().eq_ignore_ascii_case(name.as_ref()),
            Self::NameRef(name) => repo.name.as_ref().eq_ignore_ascii_case(name.as_ref()),
        }
    }
}

impl FromStr for GiteaRepoArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<GiteaRepoId>() {
            Ok(Self::Id(id))
        } else if let Ok(full_name) = s.parse::<GiteaRepoFullName>() {
            Ok(Self::FullName(full_name))
        } else if let Ok(name) = s.parse::<GiteaRepoName>() {
            Ok(Self::Name(name))
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
