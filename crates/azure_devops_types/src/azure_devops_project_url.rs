use crate::AzureDevOpsOrganizationUrl;
use crate::AzureDevOpsProjectArgument;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;
use url::Url;

const PROJECT_ROUTE: [&str; 2] = ["_apis", "projects"];

/// The REST URL of an Azure DevOps project.
///
/// See the `url` property in Microsoft's [TeamProjectReference schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/core/projects/list?view=azure-devops-rest-7.1#teamprojectreference)
/// and the [Projects Get 7.1 endpoint](https://learn.microsoft.com/en-us/rest/api/azure/devops/core/projects/get?view=azure-devops-rest-7.1).
/// The organization and project ID/name are stored as domain values. Rendering
/// constructs the route and encodes the project path segment from those values.
/// Parsing preserves the organization/collection scope and opaque query string,
/// including Azure DevOps Server HTTP URLs and ports, while canonicalizing the
/// REST route. Absolute HTTP(S), credential-free URLs and the absence of REST
/// fragments are library safety constraints.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsProjectUrl {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'static>,
    pub query: Option<String>,
}

impl AzureDevOpsProjectUrl {
    /// Build the project REST endpoint with an encoded ID/name path segment.
    pub fn new(
        organization: &AzureDevOpsOrganizationUrl,
        project: &AzureDevOpsProjectArgument<'_>,
    ) -> Result<Self> {
        let value = Self {
            org_url: organization.expanded_form().parse()?,
            project: project.to_string().parse()?,
            query: Some("api-version=7.1".to_owned()),
        };
        value.to_url()?;
        Ok(value)
    }

    pub fn org_url(&self) -> &AzureDevOpsOrganizationUrl {
        &self.org_url
    }

    pub fn project(&self) -> &AzureDevOpsProjectArgument<'static> {
        &self.project
    }

    /// Assemble the REST URL from its typed organization and project.
    pub fn to_url(&self) -> Result<Url> {
        let mut url = Url::parse(&self.org_url.expanded_form())?;
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| eyre::eyre!("Azure DevOps organization URL must be a base URL"))?;
            path.pop_if_empty()
                .extend(PROJECT_ROUTE)
                .push(&self.project.to_string());
        }
        url.set_query(self.query.as_deref());
        Ok(url)
    }

    pub fn into_url(self) -> Result<Url> {
        self.to_url()
    }
}

impl FromStr for AzureDevOpsProjectUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Url::parse(value)?.try_into()
    }
}

impl TryFrom<String> for AzureDevOpsProjectUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<&str> for AzureDevOpsProjectUrl {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<Url> for AzureDevOpsProjectUrl {
    type Error = eyre::Report;

    fn try_from(value: Url) -> Result<Self> {
        ensure!(
            value.fragment().is_none(),
            "REST project URLs must not contain fragments"
        );
        let segments = value
            .path_segments()
            .ok_or_else(|| eyre::eyre!("Azure DevOps REST project URL must have a path"))?
            .collect::<Vec<_>>();
        ensure!(
            segments.len() > PROJECT_ROUTE.len(),
            "Azure DevOps REST project identity is missing"
        );
        let route_start = segments.len() - PROJECT_ROUTE.len() - 1;
        ensure!(
            segments[route_start..segments.len() - 1]
                .iter()
                .zip(PROJECT_ROUTE)
                .all(|(actual, expected)| actual.eq_ignore_ascii_case(expected)),
            "URL does not identify an Azure DevOps REST project"
        );
        let identifier = segments[segments.len() - 1];
        ensure!(
            !identifier.is_empty(),
            "Azure DevOps project identity is missing"
        );
        // Decode a path segment without treating a literal '+' as a space.
        let mut decoded = Vec::with_capacity(identifier.len());
        let mut bytes = identifier.bytes();
        while let Some(byte) = bytes.next() {
            if byte == b'%' {
                let high = bytes
                    .next()
                    .and_then(|byte| (byte as char).to_digit(16))
                    .ok_or_else(|| eyre::eyre!("Invalid percent escape in project URL"))?;
                let low = bytes
                    .next()
                    .and_then(|byte| (byte as char).to_digit(16))
                    .ok_or_else(|| eyre::eyre!("Invalid percent escape in project URL"))?;
                decoded.push(((high << 4) | low) as u8);
            } else {
                decoded.push(byte);
            }
        }
        let project = String::from_utf8(decoded)?.parse()?;
        let mut organization = value.clone();
        organization.set_query(None);
        {
            let mut path = organization
                .path_segments_mut()
                .map_err(|_| eyre::eyre!("Azure DevOps organization URL must be a base URL"))?;
            for _ in 0..PROJECT_ROUTE.len() + 1 {
                path.pop();
            }
        }
        Ok(Self {
            org_url: organization.as_str().trim_end_matches('/').parse()?,
            project,
            query: value.query().map(str::to_owned),
        })
    }
}

impl From<&AzureDevOpsProjectUrl> for String {
    fn from(value: &AzureDevOpsProjectUrl) -> Self {
        value.to_string()
    }
}

impl TryFrom<AzureDevOpsProjectUrl> for Url {
    type Error = eyre::Report;

    fn try_from(value: AzureDevOpsProjectUrl) -> Result<Self> {
        value.into_url()
    }
}

impl TryFrom<&AzureDevOpsProjectUrl> for Url {
    type Error = eyre::Report;

    fn try_from(value: &AzureDevOpsProjectUrl) -> Result<Self> {
        value.to_url()
    }
}

impl std::fmt::Display for AzureDevOpsProjectUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url = self.to_url().map_err(|_| std::fmt::Error)?;
        f.write_str(url.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::new(
            &AzureDevOpsOrganizationUrl::arbitrary(u)?,
            &AzureDevOpsProjectArgument::arbitrary(u)?,
        )
        .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectUrl);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureDevOpsBuildDefinitionId;
    use crate::AzureDevOpsBuildDefinitionUrl;
    use crate::AzureDevOpsBuildId;
    use crate::AzureDevOpsBuildUrl;
    use crate::AzureDevOpsProjectName;

    #[test]
    fn literal_default_project_names_remain_explicit_url_identities() -> Result<()> {
        let organization = AzureDevOpsOrganizationUrl::try_new_dev_azure_com("synthetic")?;
        let project = AzureDevOpsProjectArgument::from(AzureDevOpsProjectName::try_new("default")?);
        let project_url = AzureDevOpsProjectUrl::new(&organization, &project)?;
        let build_url =
            AzureDevOpsBuildUrl::new(&organization, &project, AzureDevOpsBuildId::new(42)?)?;
        let definition_url = AzureDevOpsBuildDefinitionUrl::new(
            &organization,
            &project,
            AzureDevOpsBuildDefinitionId::new(7)?,
        )?;
        assert_eq!(project_url.project, project);
        assert_eq!(build_url.project, project);
        assert_eq!(definition_url.project, project);
        assert_eq!(
            project_url.to_string().parse::<AzureDevOpsProjectUrl>()?,
            project_url
        );
        assert_eq!(
            build_url.to_string().parse::<AzureDevOpsBuildUrl>()?,
            build_url
        );
        assert_eq!(
            definition_url
                .to_string()
                .parse::<AzureDevOpsBuildDefinitionUrl>()?,
            definition_url
        );

        Ok(())
    }

    #[test]
    fn constructs_project_routes_and_encodes_names() -> Result<()> {
        let project: AzureDevOpsProjectArgument<'static> = "Release Team Ω".parse()?;
        let organization = AzureDevOpsOrganizationUrl::try_new_dev_azure_com("synthetic")?;
        let url = AzureDevOpsProjectUrl::new(&organization, &project)?;
        assert_eq!(url.org_url(), &organization);
        assert_eq!(url.project().to_string(), "Release Team Ω");
        let assembled = url.to_url()?;
        assert_eq!(
            assembled.path(),
            "/synthetic/_apis/projects/Release%20Team%20%CE%A9"
        );
        assert_eq!(assembled.query(), Some("api-version=7.1"));

        let legacy = AzureDevOpsOrganizationUrl::try_new_visual_studio_com("synthetic")?;
        let legacy_url = AzureDevOpsProjectUrl::new(&legacy, &project)?.into_url()?;
        assert_eq!(legacy_url.host_str(), Some("synthetic.visualstudio.com"));
        assert_eq!(legacy_url.path(), "/_apis/projects/Release%20Team%20%CE%A9");

        let server =
            AzureDevOpsOrganizationUrl::try_new("http://devops.invalid:8080/tfs", "collection")?;
        let server_url = AzureDevOpsProjectUrl::new(&server, &project)?.into_url()?;
        assert_eq!(server_url.scheme(), "http");
        assert_eq!(server_url.port(), Some(8080));
        assert_eq!(
            server_url.path(),
            "/tfs/collection/_apis/projects/Release%20Team%20%CE%A9"
        );
        Ok(())
    }

    #[test]
    fn recovers_typed_service_scopes_and_preserves_query_metadata() -> Result<()> {
        for (text, canonical, organization, project) in [
            (
                "https://dev.azure.com/synthetic/_apis/projects/00000000-0000-0000-0000-000000000042?includeHistory=true&api-version=7.1",
                "https://dev.azure.com/synthetic/_apis/projects/00000000-0000-0000-0000-000000000042?includeHistory=true&api-version=7.1",
                "https://dev.azure.com/synthetic",
                "00000000-0000-0000-0000-000000000042",
            ),
            (
                "https://synthetic.visualstudio.com/DefaultCollection/_APIS/Projects/Project%20one?custom=a%2Fb",
                "https://synthetic.visualstudio.com/DefaultCollection/_apis/projects/Project%20one?custom=a%2Fb",
                "https://synthetic.visualstudio.com/DefaultCollection",
                "Project one",
            ),
            (
                "http://devops.invalid:8080/tfs/collection/_apis/projects/Project%20one",
                "http://devops.invalid:8080/tfs/collection/_apis/projects/Project%20one",
                "http://devops.invalid:8080/tfs/collection",
                "Project one",
            ),
        ] {
            let url: AzureDevOpsProjectUrl = text.parse()?;
            assert_eq!(url.org_url().to_string(), organization);
            assert_eq!(url.project().to_string(), project);
            assert_eq!(url.to_string(), canonical);
            assert_eq!(url.clone().into_url()?, Url::parse(canonical)?);
        }
        Ok(())
    }

    #[test]
    fn validates_projects_and_url_safety_during_parsing() -> Result<()> {
        for text in [
            "relative/_apis/projects/project",
            "ftp://devops.invalid/_apis/projects/project",
            "https://user:password@devops.invalid/_apis/projects/project",
            "https://devops.invalid/_apis/build/definitions/42",
            "https://devops.invalid/_apis/projects/",
            "https://devops.invalid/_apis/projects/project/extra",
            "https://devops.invalid/_apis/projects/project#fragment",
            "https://devops.invalid/_apis/projects/Project%2Fone",
            "https://devops.invalid/_apis/projects/Project%00one",
            "https://devops.invalid/_apis/projects/_InvalidName",
            "https://devops.invalid/collection/_apis/projects/Project%2Fone",
            "https://devops.invalid/collection/_apis/projects/Project%00one",
            "https://devops.invalid/collection/_apis/projects/Project%zzone",
        ] {
            assert!(text.parse::<AzureDevOpsProjectUrl>().is_err());
        }
        Ok(())
    }

    #[test]
    fn arbitrary_projects_are_valid_and_optional_values_need_no_field_overrides() -> Result<()> {
        for seed in 0..32 {
            let bytes = [seed; 1024];
            let mut u = arbitrary::Unstructured::new(&bytes);
            let url = AzureDevOpsProjectUrl::arbitrary(&mut u)?;
            assert!(url.to_url().is_ok());
            assert_eq!(url.to_string().parse::<AzureDevOpsProjectUrl>()?, url);
            let optional: Option<AzureDevOpsProjectUrl> = u.arbitrary()?;
            if let Some(url) = optional {
                assert_eq!(url.to_string().parse::<AzureDevOpsProjectUrl>()?, url);
            }
        }
        Ok(())
    }
}
