use crate::prelude::AzureDevOpsProject;
use crate::prelude::AzureDevOpsProjectId;
use crate::prelude::AzureDevOpsProjectName;

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
}
