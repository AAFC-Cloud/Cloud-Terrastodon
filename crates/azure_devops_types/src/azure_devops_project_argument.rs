use crate::prelude::AzureDevOpsProject;
use crate::prelude::AzureDevOpsProjectId;
use crate::prelude::AzureDevOpsProjectName;
use eyre::bail;
use std::str::FromStr;

pub enum AzureDevOpsProjectArgument<'a> {
    Id(AzureDevOpsProjectId),
    IdRef(&'a AzureDevOpsProjectId),
    Name(AzureDevOpsProjectName),
    NameRef(&'a AzureDevOpsProjectName),
}
impl std::fmt::Display for AzureDevOpsProjectArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsProjectArgument::Id(id) => write!(f, "{id}"),
            AzureDevOpsProjectArgument::IdRef(id) => write!(f, "{id}"),
            AzureDevOpsProjectArgument::Name(name) => write!(f, "{name}"),
            AzureDevOpsProjectArgument::NameRef(name) => write!(f, "{name}"),
        }
    }
}
impl From<AzureDevOpsProjectId> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProjectId) -> Self {
        AzureDevOpsProjectArgument::Id(value)
    }
}
impl<'a> From<&'a AzureDevOpsProjectId> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProjectId) -> Self {
        AzureDevOpsProjectArgument::IdRef(value)
    }
}
impl From<AzureDevOpsProject> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProject) -> Self {
        AzureDevOpsProjectArgument::Id(value.id)
    }
}
impl<'a> From<&'a AzureDevOpsProject> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProject) -> Self {
        AzureDevOpsProjectArgument::IdRef(&value.id)
    }
}
impl From<AzureDevOpsProjectName> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProjectName) -> Self {
        AzureDevOpsProjectArgument::Name(value)
    }
}
impl<'a> From<&'a AzureDevOpsProjectName> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProjectName) -> Self {
        AzureDevOpsProjectArgument::NameRef(value)
    }
}

impl AzureDevOpsProjectArgument<'_> {
    pub fn into_owned(self) -> AzureDevOpsProjectArgument<'static> {
        match self {
            AzureDevOpsProjectArgument::Id(id) => AzureDevOpsProjectArgument::Id(id),
            AzureDevOpsProjectArgument::IdRef(id) => AzureDevOpsProjectArgument::Id(id.clone()),
            AzureDevOpsProjectArgument::Name(name) => AzureDevOpsProjectArgument::Name(name),
            AzureDevOpsProjectArgument::NameRef(name) => {
                AzureDevOpsProjectArgument::Name(name.clone())
            }
        }
    }

    /// Returns true if this argument matches the supplied project.
    pub fn matches(&self, project: &AzureDevOpsProject) -> bool {
        match self {
            AzureDevOpsProjectArgument::Id(id) => project.id == *id,
            AzureDevOpsProjectArgument::IdRef(id) => project.id == **id,
            AzureDevOpsProjectArgument::Name(name) => {
                project.name.as_ref().eq_ignore_ascii_case(name.as_ref())
            }
            AzureDevOpsProjectArgument::NameRef(name) => {
                project.name.as_ref().eq_ignore_ascii_case(name.as_ref())
            }
        }
    }
}

impl FromStr for AzureDevOpsProjectArgument<'static> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsProjectId>() {
            Ok(AzureDevOpsProjectArgument::Id(id))
        } else if let Ok(name) = AzureDevOpsProjectName::try_new(s) {
            Ok(AzureDevOpsProjectArgument::Name(name))
        } else {
            bail!("'{s}' is not a valid Azure DevOps project id or name")
        }
    }
}
