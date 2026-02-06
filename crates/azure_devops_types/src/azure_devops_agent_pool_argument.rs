use crate::prelude::AzureDevOpsAgentPool;
use crate::prelude::AzureDevOpsAgentPoolId;
use crate::prelude::AzureDevOpsAgentPoolName;
use crate::prelude::AzureDevOpsProjectArgument;
use eyre::bail;
use std::str::FromStr;

/// The name or identifier for an [`AzureDevOpsAgentPool`]
#[derive(Debug, Clone)]
pub enum AzureDevOpsAgentPoolArgument<'a> {
    Id(AzureDevOpsAgentPoolId),
    IdRef(&'a AzureDevOpsAgentPoolId),
    Name(AzureDevOpsAgentPoolName),
    NameRef(&'a AzureDevOpsAgentPoolName),
}
impl std::fmt::Display for AzureDevOpsAgentPoolArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsAgentPoolArgument::Id(id) => id.fmt(f),
            AzureDevOpsAgentPoolArgument::IdRef(id) => id.fmt(f),
            AzureDevOpsAgentPoolArgument::Name(name) => name.fmt(f),
            AzureDevOpsAgentPoolArgument::NameRef(name) => name.fmt(f),
        }
    }
}
impl From<AzureDevOpsAgentPoolId> for AzureDevOpsAgentPoolArgument<'_> {
    fn from(value: AzureDevOpsAgentPoolId) -> Self {
        AzureDevOpsAgentPoolArgument::Id(value)
    }
}
impl<'a> From<&'a AzureDevOpsAgentPoolId> for AzureDevOpsAgentPoolArgument<'a> {
    fn from(value: &'a AzureDevOpsAgentPoolId) -> Self {
        AzureDevOpsAgentPoolArgument::IdRef(value)
    }
}
impl From<AzureDevOpsAgentPool> for AzureDevOpsAgentPoolArgument<'_> {
    fn from(value: AzureDevOpsAgentPool) -> Self {
        AzureDevOpsAgentPoolArgument::Id(value.id)
    }
}
impl<'a> From<&'a AzureDevOpsAgentPool> for AzureDevOpsAgentPoolArgument<'a> {
    fn from(value: &'a AzureDevOpsAgentPool) -> Self {
        AzureDevOpsAgentPoolArgument::IdRef(&value.id)
    }
}
impl From<AzureDevOpsAgentPoolName> for AzureDevOpsAgentPoolArgument<'_> {
    fn from(value: AzureDevOpsAgentPoolName) -> Self {
        AzureDevOpsAgentPoolArgument::Name(value)
    }
}
impl<'a> From<&'a AzureDevOpsAgentPoolName> for AzureDevOpsAgentPoolArgument<'a> {
    fn from(value: &'a AzureDevOpsAgentPoolName) -> Self {
        AzureDevOpsAgentPoolArgument::NameRef(value)
    }
}

impl AzureDevOpsAgentPoolArgument<'_> {
    pub fn into_owned(self) -> AzureDevOpsAgentPoolArgument<'static> {
        match self {
            AzureDevOpsAgentPoolArgument::Id(id) => AzureDevOpsAgentPoolArgument::Id(id),
            AzureDevOpsAgentPoolArgument::IdRef(id) => AzureDevOpsAgentPoolArgument::Id(id.clone()),
            AzureDevOpsAgentPoolArgument::Name(name) => AzureDevOpsAgentPoolArgument::Name(name),
            AzureDevOpsAgentPoolArgument::NameRef(name) => {
                AzureDevOpsAgentPoolArgument::Name(name.clone())
            }
        }
    }

    pub fn matches<'a>(&self, other: impl Into<AzureDevOpsAgentPoolArgument<'a>>) -> bool {
        let other = other.into();
        match (self, other) {
            (AzureDevOpsAgentPoolArgument::Id(id1), AzureDevOpsAgentPoolArgument::Id(id2)) => {
                *id1 == id2
            }
            (
                AzureDevOpsAgentPoolArgument::IdRef(id1),
                AzureDevOpsAgentPoolArgument::IdRef(id2),
            ) => *id1 == id2,
            (
                AzureDevOpsAgentPoolArgument::Name(name1),
                AzureDevOpsAgentPoolArgument::Name(name2),
            ) => name1.as_ref().eq_ignore_ascii_case(name2.as_ref()),
            (
                AzureDevOpsAgentPoolArgument::NameRef(name1),
                AzureDevOpsAgentPoolArgument::NameRef(name2),
            ) => name1.as_ref().eq_ignore_ascii_case(name2.as_ref()),
            _ => false,
        }
    }
}

impl FromStr for AzureDevOpsAgentPoolArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsAgentPoolId>() {
            Ok(AzureDevOpsAgentPoolArgument::Id(id))
        } else if let Ok(name) = AzureDevOpsAgentPoolName::try_new(s) {
            Ok(AzureDevOpsAgentPoolArgument::Name(name))
        } else {
            bail!("'{s}' is not a valid Azure DevOps agent pool id or name")
        }
    }
}
