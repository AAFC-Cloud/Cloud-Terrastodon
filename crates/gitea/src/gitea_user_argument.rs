use crate::GiteaUser;
use crate::GiteaUserId;
use crate::GiteaUsername;
use arbitrary::Arbitrary;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Arbitrary, Facet)]
#[repr(C)]
pub enum GiteaUserArgument<'a> {
    Id(Cow<'a, GiteaUserId>),
    Username(Cow<'a, GiteaUsername>),
}

impl Display for GiteaUserArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Username(username) => username.fmt(f),
        }
    }
}

impl From<GiteaUserId> for GiteaUserArgument<'_> {
    fn from(value: GiteaUserId) -> Self {
        Self::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaUserId> for GiteaUserArgument<'a> {
    fn from(value: &'a GiteaUserId) -> Self {
        Self::Id(Cow::Borrowed(value))
    }
}

impl From<GiteaUsername> for GiteaUserArgument<'_> {
    fn from(value: GiteaUsername) -> Self {
        Self::Username(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaUsername> for GiteaUserArgument<'a> {
    fn from(value: &'a GiteaUsername) -> Self {
        Self::Username(Cow::Borrowed(value))
    }
}

impl GiteaUserArgument<'_> {
    pub fn into_owned(self) -> GiteaUserArgument<'static> {
        match self {
            Self::Id(id) => GiteaUserArgument::Id(Cow::Owned(id.into_owned())),
            Self::Username(username) => {
                GiteaUserArgument::Username(Cow::Owned(username.into_owned()))
            }
        }
    }

    pub fn matches(&self, user: &GiteaUser) -> bool {
        match self {
            Self::Id(id) => &user.id == id.as_ref(),
            Self::Username(username) => user
                .login
                .as_ref()
                .eq_ignore_ascii_case(username.as_ref().as_ref()),
        }
    }
}

impl FromStr for GiteaUserArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<GiteaUserId>() {
            Ok(Self::Id(Cow::Owned(id)))
        } else if let Ok(username) = s.parse::<GiteaUsername>() {
            Ok(Self::Username(Cow::Owned(username)))
        } else {
            bail!("'{s}' is not a valid Gitea user id or username")
        }
    }
}

impl TryFrom<String> for GiteaUserArgument<'static> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&GiteaUserArgument<'_>> for String {
    fn from(value: &GiteaUserArgument<'_>) -> Self {
        value.to_string()
    }
}
