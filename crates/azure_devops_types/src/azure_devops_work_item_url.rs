#![cfg_attr(not(test), deny(clippy::expect_used))]

use crate::AzureDevOpsOrganizationName;
use crate::AzureDevOpsOrganizationUrl;
use crate::AzureDevOpsProjectArgument;
use crate::AzureDevOpsWorkItemId;
use crate::AzureDevOpsWorkItemProjectUrl;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use std::borrow::Cow;
use std::str::FromStr;
use url::Url;

const WORK_ITEM_ROUTE: [&str; 3] = ["_apis", "wit", "workitems"];

/// A work item URL that is scoped to an Azure DevOps organization.
///
/// The project segment is intentionally absent. Use
/// [`AzureDevOpsWorkItemUrl::with_project`] when constructing a project-scoped
/// endpoint.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsWorkItemUrl<'a> {
    pub org_id: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub work_item_id: AzureDevOpsWorkItemId,
}

impl<'a> AzureDevOpsWorkItemUrl<'a> {
    pub fn new(
        org_id: impl Into<Cow<'a, AzureDevOpsOrganizationUrl>>,
        work_item_id: AzureDevOpsWorkItemId,
    ) -> Self {
        Self {
            org_id: org_id.into(),
            work_item_id,
        }
    }

    pub fn with_project(
        self,
        project: AzureDevOpsProjectArgument<'a>,
    ) -> AzureDevOpsWorkItemProjectUrl<'a> {
        AzureDevOpsWorkItemProjectUrl {
            org_id: self.org_id,
            project,
            work_item_id: self.work_item_id,
        }
    }

    /// Returns the canonical URL with an optional project path segment.
    pub fn into_url_with_project(
        self,
        project: Option<AzureDevOpsProjectArgument<'a>>,
    ) -> Result<Url> {
        match project {
            Some(project) => self.with_project(project).into_url(),
            None => self.into_url(),
        }
    }

    /// Returns the canonical Azure DevOps REST URL for this work item.
    pub fn into_url(self) -> Result<Url> {
        build_url(&self.org_id, None, self.work_item_id)
    }

    /// Returns this URL's work item ID after verifying its organization.
    pub fn work_item_id_for_organization(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> Result<AzureDevOpsWorkItemId> {
        ensure!(
            self.org_id
                .organization_name
                .as_ref()
                .eq_ignore_ascii_case(organization.organization_name.as_ref()),
            "Relation targets another organization"
        );
        Ok(self.work_item_id)
    }
}

impl TryFrom<AzureDevOpsWorkItemUrl<'_>> for Url {
    type Error = eyre::Error;

    fn try_from(value: AzureDevOpsWorkItemUrl<'_>) -> Result<Self, Self::Error> {
        value.into_url()
    }
}

impl TryFrom<&AzureDevOpsWorkItemUrl<'_>> for Url {
    type Error = eyre::Error;

    fn try_from(value: &AzureDevOpsWorkItemUrl<'_>) -> Result<Self, Self::Error> {
        value.clone().into_url()
    }
}

impl From<&AzureDevOpsWorkItemUrl<'_>> for String {
    fn from(value: &AzureDevOpsWorkItemUrl<'_>) -> Self {
        url_string(&value.org_id, None, value.work_item_id)
    }
}

impl std::fmt::Display for AzureDevOpsWorkItemUrl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&url_string(&self.org_id, None, self.work_item_id))
    }
}

impl<'a> FromStr for AzureDevOpsWorkItemUrl<'a> {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(value)?;
        let (organization, _project, work_item_id) = parse_url(&url)?;
        Ok(Self::new(Cow::Owned(organization), work_item_id))
    }
}

impl<'a> TryFrom<String> for AzureDevOpsWorkItemUrl<'a> {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<'a> TryFrom<Url> for AzureDevOpsWorkItemUrl<'a> {
    type Error = eyre::Error;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}

impl<'a> TryFrom<&Url> for AzureDevOpsWorkItemUrl<'a> {
    type Error = eyre::Error;

    fn try_from(value: &Url) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItemUrl<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(
            Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            AzureDevOpsWorkItemId::arbitrary(u)?,
        ))
    }
}

pub(crate) fn build_url(
    organization: &AzureDevOpsOrganizationUrl,
    project: Option<&AzureDevOpsProjectArgument<'_>>,
    work_item_id: AzureDevOpsWorkItemId,
) -> Result<Url> {
    let mut url = Url::parse(&organization.to_string())?;
    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| eyre::eyre!("Azure DevOps organization URL must be a base URL"))?;
        path.pop_if_empty();
        if let Some(project) = project {
            path.push(&project.to_string());
        }
        path.extend(WORK_ITEM_ROUTE).push(&work_item_id.to_string());
    }
    url.query_pairs_mut().append_pair("api-version", "7.1");
    Ok(url)
}

fn url_string(
    organization: &AzureDevOpsOrganizationUrl,
    project: Option<&AzureDevOpsProjectArgument<'_>>,
    work_item_id: AzureDevOpsWorkItemId,
) -> String {
    match build_url(organization, project, work_item_id) {
        Ok(url) => url.into(),
        Err(_) => {
            let project = project
                .map(|project| format!("{project}/"))
                .unwrap_or_default();
            format!(
                "{}/{project}_apis/wit/workitems/{work_item_id}?api-version=7.1",
                organization.expanded_form().trim_end_matches('/')
            )
        }
    }
}

pub(crate) fn parse_url(
    url: &Url,
) -> Result<(
    AzureDevOpsOrganizationUrl,
    Option<AzureDevOpsProjectArgument<'static>>,
    AzureDevOpsWorkItemId,
)> {
    ensure!(
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none(),
        "Invalid Azure DevOps work item URL"
    );
    let segments: Vec<_> = url
        .path_segments()
        .ok_or_else(|| eyre::eyre!("Invalid Azure DevOps work item path"))?
        .collect();
    let (organization, mut tail) = if url.host_str() == Some("dev.azure.com") {
        let organization: AzureDevOpsOrganizationName = segments
            .first()
            .ok_or_else(|| eyre::eyre!("Organization is missing from work item URL"))?
            .parse()?;
        (
            AzureDevOpsOrganizationUrl::new_dev_azure_com(organization),
            &segments[1..],
        )
    } else if let Some(host) = url.host_str() {
        let suffix = ".visualstudio.com";
        let organization_name: AzureDevOpsOrganizationName = host
            .strip_suffix(suffix)
            .ok_or_else(|| eyre::eyre!("Unsupported Azure DevOps work item host"))?
            .parse()?;
        let tail = if segments.first() == Some(&"DefaultCollection") {
            &segments[1..]
        } else {
            &segments[..]
        };
        (
            AzureDevOpsOrganizationUrl::new_visual_studio_com(organization_name),
            tail,
        )
    } else {
        bail!("Azure DevOps work item URL has no host")
    };
    ensure!(
        tail.len() == WORK_ITEM_ROUTE.len() + 1 || tail.len() == WORK_ITEM_ROUTE.len() + 2,
        "Invalid Azure DevOps work item path"
    );
    let route_start = tail.len() - (WORK_ITEM_ROUTE.len() + 1);
    let project = if route_start == 1 {
        Some(tail[0].parse::<AzureDevOpsProjectArgument<'static>>()?)
    } else {
        None
    };
    tail = &tail[route_start..];
    ensure!(
        tail[..WORK_ITEM_ROUTE.len()]
            .iter()
            .zip(WORK_ITEM_ROUTE)
            .all(|(actual, expected)| actual.eq_ignore_ascii_case(expected)),
        "Invalid Azure DevOps work item route"
    );
    let work_item_id = tail[WORK_ITEM_ROUTE.len()].parse()?;
    Ok((organization, project, work_item_id))
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemUrl<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemUrl<'static>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_builds_organization_work_item_urls() -> Result<()> {
        let url: AzureDevOpsWorkItemUrl<'static> =
            "https://dev.azure.com/example/_apis/wit/workitems/42".parse()?;
        assert_eq!(url.work_item_id.get(), 42);
        assert_eq!(
            url.clone().into_url()?.path(),
            "/example/_apis/wit/workitems/42"
        );
        Ok(())
    }
}
