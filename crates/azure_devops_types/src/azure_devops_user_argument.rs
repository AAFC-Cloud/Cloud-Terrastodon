use crate::AzureDevOpsLicenseEntitlementUserReference;
use crate::AzureDevOpsUserId;
use crate::AzureDevOpsUserLicenseEntitlement;
use arbitrary::Arbitrary;
use eyre::bail;
use std::borrow::Cow;
use std::str::FromStr;

type AzureDevOpsUserPredicate<'a> = dyn Fn(&AzureDevOpsUserLicenseEntitlement) -> bool + 'a;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsUserArgument<'a> {
    Id(Cow<'a, AzureDevOpsUserId>),
    Email(Cow<'a, str>),
}

impl std::fmt::Display for AzureDevOpsUserArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsUserArgument::Id(id) => write!(f, "{id}"),
            AzureDevOpsUserArgument::Email(name) => write!(f, "{name}"),
        }
    }
}

impl From<AzureDevOpsUserId> for AzureDevOpsUserArgument<'_> {
    fn from(value: AzureDevOpsUserId) -> Self {
        AzureDevOpsUserArgument::Id(Cow::Owned(value))
    }
}

impl<'a> From<&'a AzureDevOpsUserId> for AzureDevOpsUserArgument<'a> {
    fn from(value: &'a AzureDevOpsUserId) -> Self {
        AzureDevOpsUserArgument::Id(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a AzureDevOpsUserArgument<'a>> for AzureDevOpsUserArgument<'a> {
    fn from(value: &'a AzureDevOpsUserArgument<'a>) -> Self {
        value.clone()
    }
}

impl AzureDevOpsUserArgument<'_> {
    pub fn as_predicate<'a>(&'a self) -> eyre::Result<Box<AzureDevOpsUserPredicate<'a>>> {
        Ok(Box::new(move |e| self.matches(&e.user)))
    }

    pub fn into_owned(self) -> AzureDevOpsUserArgument<'static> {
        match self {
            AzureDevOpsUserArgument::Id(id) => {
                AzureDevOpsUserArgument::Id(Cow::Owned(id.into_owned()))
            }
            AzureDevOpsUserArgument::Email(email) => {
                AzureDevOpsUserArgument::Email(Cow::Owned(email.into_owned()))
            }
        }
    }

    pub fn matches(&self, user: &AzureDevOpsLicenseEntitlementUserReference) -> bool {
        match self {
            AzureDevOpsUserArgument::Id(id) => &user.id == id.as_ref(),
            AzureDevOpsUserArgument::Email(email) => {
                user.unique_name.eq_ignore_ascii_case(email.as_ref())
            }
        }
    }
}

impl<'a> FromStr for AzureDevOpsUserArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsUserId>() {
            Ok(AzureDevOpsUserArgument::Id(Cow::Owned(id)))
        } else if s.contains('@') {
            Ok(AzureDevOpsUserArgument::Email(Cow::Owned(s.to_string())))
        } else {
            bail!("'{s}' is not a valid Azure DevOps user id or email")
        }
    }
}

impl<'a> TryFrom<String> for AzureDevOpsUserArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsUserArgument<'_>> for String {
    fn from(value: &AzureDevOpsUserArgument<'_>) -> Self {
        value.to_string()
    }
}
