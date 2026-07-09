use crate::AzureDevOpsAgentPool;
use crate::AzureDevOpsAgentPoolEntitlement;
use crate::AzureDevOpsAgentPoolId;
use crate::AzureDevOpsAgentPoolName;
use arbitrary::Arbitrary;
use eyre::bail;
use std::borrow::Cow;
use std::str::FromStr;

/// The name or identifier for an [`AzureDevOpsAgentPool`]
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsAgentPoolArgument<'a> {
    Id(Cow<'a, AzureDevOpsAgentPoolId>),
    Name(Cow<'a, AzureDevOpsAgentPoolName>),
}
impl std::fmt::Display for AzureDevOpsAgentPoolArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsAgentPoolArgument::Id(id) => id.fmt(f),
            AzureDevOpsAgentPoolArgument::Name(name) => name.fmt(f),
        }
    }
}
impl From<AzureDevOpsAgentPoolId> for AzureDevOpsAgentPoolArgument<'_> {
    fn from(value: AzureDevOpsAgentPoolId) -> Self {
        AzureDevOpsAgentPoolArgument::Id(Cow::Owned(value))
    }
}
impl<'a> From<&'a AzureDevOpsAgentPoolId> for AzureDevOpsAgentPoolArgument<'a> {
    fn from(value: &'a AzureDevOpsAgentPoolId) -> Self {
        AzureDevOpsAgentPoolArgument::Id(Cow::Borrowed(value))
    }
}
impl From<AzureDevOpsAgentPool> for AzureDevOpsAgentPoolArgument<'_> {
    fn from(value: AzureDevOpsAgentPool) -> Self {
        AzureDevOpsAgentPoolArgument::Id(Cow::Owned(value.id))
    }
}
impl<'a> From<&'a AzureDevOpsAgentPool> for AzureDevOpsAgentPoolArgument<'a> {
    fn from(value: &'a AzureDevOpsAgentPool) -> Self {
        AzureDevOpsAgentPoolArgument::Id(Cow::Borrowed(&value.id))
    }
}
impl From<AzureDevOpsAgentPoolName> for AzureDevOpsAgentPoolArgument<'_> {
    fn from(value: AzureDevOpsAgentPoolName) -> Self {
        AzureDevOpsAgentPoolArgument::Name(Cow::Owned(value))
    }
}
impl<'a> From<&'a AzureDevOpsAgentPoolName> for AzureDevOpsAgentPoolArgument<'a> {
    fn from(value: &'a AzureDevOpsAgentPoolName) -> Self {
        AzureDevOpsAgentPoolArgument::Name(Cow::Borrowed(value))
    }
}

impl AzureDevOpsAgentPoolArgument<'_> {
    pub fn into_owned(self) -> AzureDevOpsAgentPoolArgument<'static> {
        match self {
            AzureDevOpsAgentPoolArgument::Id(id) => {
                AzureDevOpsAgentPoolArgument::Id(Cow::Owned(id.into_owned()))
            }
            AzureDevOpsAgentPoolArgument::Name(name) => {
                AzureDevOpsAgentPoolArgument::Name(Cow::Owned(name.into_owned()))
            }
        }
    }

    pub fn as_id(&self) -> Option<&AzureDevOpsAgentPoolId> {
        match self {
            AzureDevOpsAgentPoolArgument::Id(id) => Some(id.as_ref()),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&AzureDevOpsAgentPoolName> {
        match self {
            AzureDevOpsAgentPoolArgument::Name(name) => Some(name.as_ref()),
            _ => None,
        }
    }

    pub fn matches<'a>(&self, other: impl Into<AzureDevOpsAgentPoolArgument<'a>>) -> bool {
        let other = other.into();
        match (self.as_id(), other.as_id(), self.as_name(), other.as_name()) {
            (Some(id1), Some(id2), _, _) => id1 == id2,
            (_, _, Some(name1), Some(name2)) => name1.as_ref().eq_ignore_ascii_case(name2.as_ref()),
            _ => false,
        }
    }
    pub fn matches_entitlement(&self, entitlement: &AzureDevOpsAgentPoolEntitlement) -> bool {
        self.matches(entitlement.pool.id) || self.matches(&entitlement.pool.name)
    }
}

impl<'a> FromStr for AzureDevOpsAgentPoolArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsAgentPoolId>() {
            Ok(AzureDevOpsAgentPoolArgument::Id(Cow::Owned(id)))
        } else if let Ok(name) = AzureDevOpsAgentPoolName::try_new(s) {
            Ok(AzureDevOpsAgentPoolArgument::Name(Cow::Owned(name)))
        } else {
            bail!("'{s}' is not a valid Azure DevOps agent pool id or name")
        }
    }
}

impl<'a> TryFrom<String> for AzureDevOpsAgentPoolArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsAgentPoolArgument<'_>> for String {
    fn from(value: &AzureDevOpsAgentPoolArgument<'_>) -> Self {
        value.to_string()
    }
}
