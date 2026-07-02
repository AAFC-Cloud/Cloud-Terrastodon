use crate::AzureDevOpsProject;
use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use eyre::bail;
use std::str::FromStr;

/// Project ID or name
#[derive(Debug, Clone, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsProjectArgument<'a> {
    Id(AzureDevOpsProjectId),
    IdRef(&'a AzureDevOpsProjectId),
    Name(AzureDevOpsProjectName),
    NameRef(&'a AzureDevOpsProjectName),
}
impl std::fmt::Display for AzureDevOpsProjectArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsProjectArgument::Id(id) => id.fmt(f),
            AzureDevOpsProjectArgument::IdRef(id) => id.fmt(f),
            AzureDevOpsProjectArgument::Name(name) => name.fmt(f),
            AzureDevOpsProjectArgument::NameRef(name) => name.fmt(f),
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

impl<'a> FromStr for AzureDevOpsProjectArgument<'a> {
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

impl<'a> TryFrom<String> for AzureDevOpsProjectArgument<'a> {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<&AzureDevOpsProjectArgument<'_>> for String {
    fn from(value: &AzureDevOpsProjectArgument<'_>) -> Self {
        value.to_string()
    }
}
