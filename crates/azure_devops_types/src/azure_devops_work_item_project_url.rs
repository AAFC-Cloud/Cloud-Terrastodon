use crate::AzureDevOpsOrganizationUrl;
use crate::AzureDevOpsProjectArgument;
use crate::AzureDevOpsWorkItemId;
use crate::AzureDevOpsWorkItemUrl;
use crate::azure_devops_work_item_url::build_url;
use crate::azure_devops_work_item_url::parse_url;
use arbitrary::Arbitrary;
use eyre::Result;
use std::borrow::Cow;
use std::str::FromStr;
use url::Url;

/// A work item URL that includes a project path segment.
#[derive(Debug, Clone, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsWorkItemProjectUrl<'a> {
    pub org_id: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub work_item_id: AzureDevOpsWorkItemId,
}

impl<'a> AzureDevOpsWorkItemProjectUrl<'a> {
    /// Returns the organization-scoped form, dropping the project path.
    pub fn without_project(self) -> AzureDevOpsWorkItemUrl<'a> {
        AzureDevOpsWorkItemUrl {
            org_id: self.org_id,
            work_item_id: self.work_item_id,
        }
    }

    /// Returns the canonical project-scoped Azure DevOps REST URL.
    pub fn into_url(self) -> Result<Url> {
        build_url(&self.org_id, Some(&self.project), self.work_item_id)
    }
}

impl TryFrom<AzureDevOpsWorkItemProjectUrl<'_>> for Url {
    type Error = eyre::Error;

    fn try_from(value: AzureDevOpsWorkItemProjectUrl<'_>) -> Result<Self, Self::Error> {
        value.into_url()
    }
}

impl TryFrom<&AzureDevOpsWorkItemProjectUrl<'_>> for Url {
    type Error = eyre::Error;

    fn try_from(value: &AzureDevOpsWorkItemProjectUrl<'_>) -> Result<Self, Self::Error> {
        value.clone().into_url()
    }
}

impl From<&AzureDevOpsWorkItemProjectUrl<'_>> for String {
    fn from(value: &AzureDevOpsWorkItemProjectUrl<'_>) -> Self {
        match build_url(&value.org_id, Some(&value.project), value.work_item_id) {
            Ok(url) => url.into(),
            Err(_) => format!(
                "{}/{}/_apis/wit/workitems/{}?api-version=7.1",
                value.org_id.expanded_form().trim_end_matches('/'),
                value.project,
                value.work_item_id
            ),
        }
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemProjectUrl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(String::from(self).as_str())
    }
}

impl<'a> FromStr for AzureDevOpsWorkItemProjectUrl<'a> {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(value)?;
        let (organization, project, work_item_id) = parse_url(&url)?;
        let project = project.ok_or_else(|| eyre::eyre!("Work item URL has no project"))?;
        Ok(Self {
            org_id: Cow::Owned(organization),
            project: project.into_owned(),
            work_item_id,
        })
    }
}

impl<'a> TryFrom<String> for AzureDevOpsWorkItemProjectUrl<'a> {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<'a> TryFrom<Url> for AzureDevOpsWorkItemProjectUrl<'a> {
    type Error = eyre::Error;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemProjectUrl<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_id: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            work_item_id: AzureDevOpsWorkItemId::arbitrary(u)?,
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemProjectUrl<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemProjectUrl<'static>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_urls_preserve_project_information() -> Result<()> {
        let url: AzureDevOpsWorkItemProjectUrl<'static> =
            "https://dev.azure.com/example/project/_apis/wit/workitems/42".parse()?;
        assert_eq!(url.work_item_id.get(), 42);
        assert_eq!(url.project.to_string(), "project");
        assert_eq!(
            url.clone().into_url()?.path(),
            "/example/project/_apis/wit/workitems/42"
        );
        Ok(())
    }
}
