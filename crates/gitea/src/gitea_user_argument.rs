use crate::GiteaUser;
use crate::GiteaUserId;
use crate::GiteaUsername;
use eyre::bail;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum GiteaUserArgument<'a> {
    Id(GiteaUserId),
    IdRef(&'a GiteaUserId),
    Username(GiteaUsername),
    UsernameRef(&'a GiteaUsername),
}

impl Display for GiteaUserArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::IdRef(id) => id.fmt(f),
            Self::Username(username) => username.fmt(f),
            Self::UsernameRef(username) => username.fmt(f),
        }
    }
}

impl GiteaUserArgument<'_> {
    pub fn matches(&self, user: &GiteaUser) -> bool {
        match self {
            Self::Id(id) => user.id == *id,
            Self::IdRef(id) => user.id == **id,
            Self::Username(username) => user.login.as_ref().eq_ignore_ascii_case(username.as_ref()),
            Self::UsernameRef(username) => {
                user.login.as_ref().eq_ignore_ascii_case(username.as_ref())
            }
        }
    }
}

impl FromStr for GiteaUserArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<GiteaUserId>() {
            Ok(Self::Id(id))
        } else if let Ok(username) = s.parse::<GiteaUsername>() {
            Ok(Self::Username(username))
        } else {
            bail!("'{s}' is not a valid Gitea user id or username")
        }
    }
}
