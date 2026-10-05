use crate::AzureDevOpsBuildId;
use crate::AzureDevOpsOrganizationUrl;
use crate::AzureDevOpsProjectArgument;
use crate::AzureDevOpsProjectId;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;
use url::Url;

const BUILD_ROUTE: [&str; 3] = ["_apis", "build", "builds"];

/// A project-scoped Azure DevOps build REST URL.
///
/// See the `url` property in Microsoft's [Build schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#build)
/// and [Builds Get 7.1](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/get?view=azure-devops-rest-7.1).
/// Organization, project, and resource identity are the stored domain values;
/// the transport URL is assembled from them. Parsing retains query parameters
/// and collection prefixes, while rendering canonicalizes route spelling and
/// path encoding. Organization forms follow [`AzureDevOpsOrganizationUrl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsBuildUrl {
    pub org_url: AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'static>,
    pub build_id: AzureDevOpsBuildId,
    pub query: Option<String>,
}

impl AzureDevOpsBuildUrl {
    /// Construct a project-scoped REST URL using API version 7.1.
    pub fn new(
        organization: &AzureDevOpsOrganizationUrl,
        project: &AzureDevOpsProjectArgument<'_>,
        build_id: AzureDevOpsBuildId,
    ) -> Result<Self> {
        let value = Self {
            org_url: organization.expanded_form().parse()?,
            project: project.to_string().parse()?,
            build_id,
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

    pub fn build_id(&self) -> AzureDevOpsBuildId {
        self.build_id
    }

    /// Assemble an encoded URL at the transport boundary.
    pub fn to_url(&self) -> Result<Url> {
        let mut url = Url::parse(&self.org_url.expanded_form())?;
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| eyre::eyre!("Azure DevOps organization URL must be a base URL"))?;
            path.pop_if_empty()
                .push(&self.project.to_string())
                .extend(BUILD_ROUTE)
                .push(&self.build_id.to_string());
        }
        url.set_query(self.query.as_deref());
        Ok(url)
    }

    pub fn into_url(self) -> Result<Url> {
        self.to_url()
    }
}

impl FromStr for AzureDevOpsBuildUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Url::parse(value)?.try_into()
    }
}

impl TryFrom<Url> for AzureDevOpsBuildUrl {
    type Error = eyre::Report;

    fn try_from(url: Url) -> Result<Self> {
        ensure!(
            url.fragment().is_none(),
            "REST resource URLs must not contain fragments"
        );
        let segments: Vec<_> = url
            .path_segments()
            .ok_or_else(|| eyre::eyre!("Azure DevOps REST URL must have a path"))?
            .collect();
        let route_start = segments
            .len()
            .checked_sub(BUILD_ROUTE.len() + 1)
            .filter(|start| *start > 0)
            .ok_or_else(|| eyre::eyre!("Project and resource identity are missing from URL"))?;
        ensure!(
            segments[route_start..segments.len() - 1]
                .iter()
                .zip(BUILD_ROUTE)
                .all(|(actual, expected)| actual.eq_ignore_ascii_case(expected)),
            "URL does not identify an Azure DevOps build"
        );
        let build_id = decode_path_segment(segments[segments.len() - 1])?.parse()?;
        let project = decode_path_segment(segments[route_start - 1])?.parse()?;

        let query = url.query().map(str::to_owned);
        let mut organization = url;
        organization.set_query(None);
        organization.set_fragment(None);
        {
            let mut path = organization
                .path_segments_mut()
                .map_err(|_| eyre::eyre!("Azure DevOps organization URL must be a base URL"))?;
            for _ in 0..BUILD_ROUTE.len() + 2 {
                path.pop();
            }
        }
        let org_url = organization.as_str().parse()?;
        Ok(Self {
            org_url,
            project,
            build_id,
            query,
        })
    }
}

impl TryFrom<String> for AzureDevOpsBuildUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<&str> for AzureDevOpsBuildUrl {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl From<&AzureDevOpsBuildUrl> for String {
    fn from(value: &AzureDevOpsBuildUrl) -> Self {
        value.to_string()
    }
}

impl TryFrom<AzureDevOpsBuildUrl> for Url {
    type Error = eyre::Report;

    fn try_from(value: AzureDevOpsBuildUrl) -> Result<Self> {
        value.into_url()
    }
}

impl TryFrom<&AzureDevOpsBuildUrl> for Url {
    type Error = eyre::Report;

    fn try_from(value: &AzureDevOpsBuildUrl) -> Result<Self> {
        value.to_url()
    }
}

impl std::fmt::Display for AzureDevOpsBuildUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url = self.to_url().map_err(|_| std::fmt::Error)?;
        url.fmt(f)
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Self::new(
            &AzureDevOpsOrganizationUrl::arbitrary(u)?,
            &AzureDevOpsProjectArgument::from(AzureDevOpsProjectId::arbitrary(u)?),
            AzureDevOpsBuildId::arbitrary(u)?,
        )
        .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

/// Decode path data literally; a '+' in a path is not a form-encoded space.
fn decode_path_segment(segment: &str) -> Result<String> {
    let mut decoded = Vec::with_capacity(segment.len());
    let mut bytes = segment.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes
                .next()
                .and_then(|byte| (byte as char).to_digit(16))
                .ok_or_else(|| eyre::eyre!("Invalid percent escape in URL path segment"))?;
            let low = bytes
                .next()
                .and_then(|byte| (byte as char).to_digit(16))
                .ok_or_else(|| eyre::eyre!("Invalid percent escape in URL path segment"))?;
            decoded.push(((high << 4) | low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    Ok(String::from_utf8(decoded)?)
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildUrl);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureDevOpsProjectName;
    use arbitrary::Unstructured;

    #[test]
    fn parses_service_forms_and_canonicalizes_resource_routes() -> Result<()> {
        for (value, canonical, organization, project) in [
            (
                "https://dev.azure.com/synthetic/Synthetic%20Project/_apis/build/builds/42?api-version=7.1",
                "https://dev.azure.com/synthetic/Synthetic%20Project/_apis/build/builds/42?api-version=7.1",
                "https://dev.azure.com/synthetic",
                "Synthetic Project",
            ),
            (
                "https://synthetic.visualstudio.com/DefaultCollection/project/_apis/build/Builds/42",
                "https://synthetic.visualstudio.com/DefaultCollection/project/_apis/build/builds/42",
                "https://synthetic.visualstudio.com/DefaultCollection",
                "project",
            ),
            (
                "http://server.example.invalid:8080/tfs/Collection/project/_apis/build/builds/42?synthetic=a%2Fb",
                "http://server.example.invalid:8080/tfs/Collection/project/_apis/build/builds/42?synthetic=a%2Fb",
                "http://server.example.invalid:8080/tfs/Collection",
                "project",
            ),
            (
                "https://builds.example.invalid/collection/project/_apis/build/builds/%34%32",
                "https://builds.example.invalid/collection/project/_apis/build/builds/42",
                "https://builds.example.invalid/collection",
                "project",
            ),
        ] {
            let url: AzureDevOpsBuildUrl = value.parse()?;
            assert_eq!(url.build_id().get(), 42);
            assert_eq!(url.org_url().to_string(), organization);
            assert_eq!(url.project().to_string(), project);
            assert_eq!(url.to_url()?.query(), Url::parse(value)?.query());
            assert_eq!(url.to_string(), canonical);
        }
        Ok(())
    }

    #[test]
    fn rejects_other_resources_and_invalid_identities_during_parsing() {
        for value in [
            "relative",
            "ftp://builds.example.invalid/project/_apis/build/builds/42",
            "https://user:password@builds.example.invalid/project/_apis/build/builds/42",
            "https://dev.azure.com/synthetic/project/_build/results?buildId=42",
            "https://dev.azure.com/synthetic/project/_apis/build/definitions/42",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/42/logs",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/0",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/-1",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/2147483648",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/%GG",
            "https://dev.azure.com/synthetic/project/_apis/build/builds/42#fragment",
        ] {
            assert!(value.parse::<AzureDevOpsBuildUrl>().is_err());
        }
    }

    #[test]
    fn constructor_encodes_project_scope_without_changing_identity() -> Result<()> {
        let project = AzureDevOpsProjectArgument::from(AzureDevOpsProjectName::try_new(
            "Synthetic Project — CI",
        )?);
        for (organization, expected_prefix) in [
            (
                "https://dev.azure.com/synthetic".parse::<AzureDevOpsOrganizationUrl>()?,
                "/synthetic/",
            ),
            (
                "https://synthetic.visualstudio.com".parse::<AzureDevOpsOrganizationUrl>()?,
                "/",
            ),
        ] {
            let url =
                AzureDevOpsBuildUrl::new(&organization, &project, AzureDevOpsBuildId::new(42)?)?;
            assert_eq!(
                url.to_url()?.path(),
                format!(
                    "{expected_prefix}Synthetic%20Project%20%E2%80%94%20CI/_apis/build/builds/42"
                )
            );
            assert_eq!(url.to_url()?.query(), Some("api-version=7.1"));
            assert_eq!(url.to_string().parse::<AzureDevOpsBuildUrl>()?, url);
        }
        Ok(())
    }

    #[test]
    fn arbitrary_generation_produces_valid_owned_domain_urls() -> Result<()> {
        for seed in 0..=u8::MAX {
            let bytes = [seed; 256];
            let value = AzureDevOpsBuildUrl::arbitrary(&mut Unstructured::new(&bytes))?;
            assert!(value.build_id().get() > 0);
            assert_eq!(value.to_string().parse::<AzureDevOpsBuildUrl>()?, value);
        }
        Ok(())
    }
}
