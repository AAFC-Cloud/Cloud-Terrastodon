use crate::AzureDevOpsLicenseEntitlementUserReference;
use crate::AzureDevOpsUserId;
use crate::AzureDevOpsUserLicenseEntitlement;
use compact_str::CompactString;
use eyre::bail;
use std::str::FromStr;

#[derive(Debug, Clone, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsUserArgument<'a> {
    Id(AzureDevOpsUserId),
    IdRef(&'a AzureDevOpsUserId),
    Email(CompactString),
    EmailRef(&'a str),
}

impl std::fmt::Display for AzureDevOpsUserArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsUserArgument::Id(id) => write!(f, "{id}"),
            AzureDevOpsUserArgument::IdRef(id) => write!(f, "{id}"),
            AzureDevOpsUserArgument::Email(name) => write!(f, "{name}"),
            AzureDevOpsUserArgument::EmailRef(name) => write!(f, "{name}"),
        }
    }
}

impl From<AzureDevOpsUserId> for AzureDevOpsUserArgument<'_> {
    fn from(value: AzureDevOpsUserId) -> Self {
        AzureDevOpsUserArgument::Id(value)
    }
}

impl<'a> From<&'a AzureDevOpsUserId> for AzureDevOpsUserArgument<'a> {
    fn from(value: &'a AzureDevOpsUserId) -> Self {
        AzureDevOpsUserArgument::IdRef(value)
    }
}

impl<'a> From<&'a AzureDevOpsUserArgument<'a>> for AzureDevOpsUserArgument<'a> {
    fn from(value: &'a AzureDevOpsUserArgument<'a>) -> Self {
        match value {
            AzureDevOpsUserArgument::Id(id) => AzureDevOpsUserArgument::Id(*id),
            AzureDevOpsUserArgument::IdRef(id) => AzureDevOpsUserArgument::IdRef(*id),
            AzureDevOpsUserArgument::Email(email) => {
                AzureDevOpsUserArgument::EmailRef(email.as_str())
            }
            AzureDevOpsUserArgument::EmailRef(email) => {
                AzureDevOpsUserArgument::EmailRef(*email)
            }
        }
    }
}

impl AzureDevOpsUserArgument<'_> {
    pub fn as_predicate<'a>(
        &'a self,
    ) -> eyre::Result<Box<dyn Fn(&AzureDevOpsUserLicenseEntitlement) -> bool + 'a>> {
        Ok(Box::new(move |e| self.matches(&e.user)))
    }

    pub fn into_owned(self) -> AzureDevOpsUserArgument<'static> {
        match self {
            AzureDevOpsUserArgument::Id(id) => AzureDevOpsUserArgument::Id(id),
            AzureDevOpsUserArgument::IdRef(id) => AzureDevOpsUserArgument::Id(*id),
            AzureDevOpsUserArgument::Email(email) => AzureDevOpsUserArgument::Email(email),
            AzureDevOpsUserArgument::EmailRef(email) => {
                AzureDevOpsUserArgument::Email(email.into())
            }
        }
    }

    pub fn matches(&self, user: &AzureDevOpsLicenseEntitlementUserReference) -> bool {
        match self {
            AzureDevOpsUserArgument::Id(id) => user.id == *id,
            AzureDevOpsUserArgument::IdRef(id) => user.id == **id,
            AzureDevOpsUserArgument::Email(email) => {
                user.unique_name.eq_ignore_ascii_case(email.as_ref())
            }
            AzureDevOpsUserArgument::EmailRef(email) => {
                user.unique_name.eq_ignore_ascii_case(email.as_ref())
            }
        }
    }
}

impl<'a> FromStr for AzureDevOpsUserArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsUserId>() {
            Ok(AzureDevOpsUserArgument::Id(id))
        } else if s.contains('@') {
            Ok(AzureDevOpsUserArgument::Email(s.into()))
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
