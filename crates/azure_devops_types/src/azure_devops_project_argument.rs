use crate::AzureDevOpsProject;
use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use arbitrary::Arbitrary;
use eyre::bail;
use std::borrow::Cow;
use std::str::FromStr;

/// Project ID or name
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureDevOpsProjectArgument<'a> {
    Id(Cow<'a, AzureDevOpsProjectId>),
    Name(Cow<'a, AzureDevOpsProjectName>),
}
impl std::fmt::Display for AzureDevOpsProjectArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsProjectArgument::Id(id) => id.fmt(f),
            AzureDevOpsProjectArgument::Name(name) => name.fmt(f),
        }
    }
}
impl From<AzureDevOpsProjectId> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProjectId) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Owned(value))
    }
}
impl<'a> From<&'a AzureDevOpsProjectId> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProjectId) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Borrowed(value))
    }
}
impl From<AzureDevOpsProject> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProject) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Owned(value.id))
    }
}
impl<'a> From<&'a AzureDevOpsProject> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProject) -> Self {
        AzureDevOpsProjectArgument::Id(Cow::Borrowed(&value.id))
    }
}
impl From<AzureDevOpsProjectName> for AzureDevOpsProjectArgument<'_> {
    fn from(value: AzureDevOpsProjectName) -> Self {
        AzureDevOpsProjectArgument::Name(Cow::Owned(value))
    }
}
impl<'a> From<&'a AzureDevOpsProjectName> for AzureDevOpsProjectArgument<'a> {
    fn from(value: &'a AzureDevOpsProjectName) -> Self {
        AzureDevOpsProjectArgument::Name(Cow::Borrowed(value))
    }
}

impl AzureDevOpsProjectArgument<'_> {
    pub fn into_owned(self) -> AzureDevOpsProjectArgument<'static> {
        match self {
            AzureDevOpsProjectArgument::Id(id) => {
                AzureDevOpsProjectArgument::Id(Cow::Owned(id.into_owned()))
            }
            AzureDevOpsProjectArgument::Name(name) => {
                AzureDevOpsProjectArgument::Name(Cow::Owned(name.into_owned()))
            }
        }
    }

    /// Returns true if this argument matches the supplied project.
    pub fn matches(&self, project: &AzureDevOpsProject) -> bool {
        match self {
            AzureDevOpsProjectArgument::Id(id) => &project.id == id.as_ref(),
            AzureDevOpsProjectArgument::Name(name) => project
                .name
                .as_ref()
                .eq_ignore_ascii_case(name.as_ref().as_ref()),
        }
    }
}
impl<'a> FromStr for AzureDevOpsProjectArgument<'a> {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<AzureDevOpsProjectId>() {
            Ok(AzureDevOpsProjectArgument::Id(Cow::Owned(id)))
        } else if let Ok(name) = AzureDevOpsProjectName::try_new(s) {
            Ok(AzureDevOpsProjectArgument::Name(Cow::Owned(name)))
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
