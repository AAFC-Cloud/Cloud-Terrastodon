use crate::GiteaInstanceUrl;
use crate::GiteaTenantAlias;
use arbitrary::Arbitrary;
use eyre::bail;
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum GiteaTenantArgument<'a> {
    #[default]
    Default,
    Url(Cow<'a, GiteaInstanceUrl>),
    Alias(Cow<'a, GiteaTenantAlias>),
}

impl Display for GiteaTenantArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GiteaTenantArgument::Default => f.write_str("default"),
            GiteaTenantArgument::Url(url) => url.fmt(f),
            GiteaTenantArgument::Alias(alias) => alias.fmt(f),
        }
    }
}

impl From<GiteaInstanceUrl> for GiteaTenantArgument<'_> {
    fn from(value: GiteaInstanceUrl) -> Self {
        Self::Url(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaInstanceUrl> for GiteaTenantArgument<'a> {
    fn from(value: &'a GiteaInstanceUrl) -> Self {
        Self::Url(Cow::Borrowed(value))
    }
}

impl From<GiteaTenantAlias> for GiteaTenantArgument<'_> {
    fn from(value: GiteaTenantAlias) -> Self {
        Self::Alias(Cow::Owned(value))
    }
}

impl<'a> From<&'a GiteaTenantAlias> for GiteaTenantArgument<'a> {
    fn from(value: &'a GiteaTenantAlias) -> Self {
        Self::Alias(Cow::Borrowed(value))
    }
}

impl GiteaTenantArgument<'_> {
    pub fn into_owned(self) -> GiteaTenantArgument<'static> {
        match self {
            GiteaTenantArgument::Default => GiteaTenantArgument::Default,
            GiteaTenantArgument::Url(url) => GiteaTenantArgument::Url(Cow::Owned(url.into_owned())),
            GiteaTenantArgument::Alias(alias) => {
                GiteaTenantArgument::Alias(Cow::Owned(alias.into_owned()))
            }
        }
    }
}

impl<'a> FromStr for GiteaTenantArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("default") {
            Ok(Self::Default)
        } else if let Ok(url) = s.parse::<GiteaInstanceUrl>() {
            Ok(Self::Url(Cow::Owned(url)))
        } else if let Ok(alias) = s.parse::<GiteaTenantAlias>() {
            Ok(Self::Alias(Cow::Owned(alias)))
        } else {
            bail!(
                "'{s}' is not a valid default selector, tracked Gitea instance URL, or Cloud Terrastodon tenant alias"
            )
        }
    }
}

impl<'a> TryFrom<String> for GiteaTenantArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&GiteaTenantArgument<'_>> for String {
    fn from(value: &GiteaTenantArgument<'_>) -> Self {
        value.to_string()
    }
}
