use crate::AzureDevOpsOrganizationName;
use crate::AzureDevOpsProjectCollectionName;
use crate::azure_devops_organization_url_name::AzureDevOpsOrganizationUrlName;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use std::str::FromStr;
use url::Url;

/// An Azure DevOps Services organization or Server collection URL.
///
/// Preserves modern `dev.azure.com`, legacy `visualstudio.com` (including a
/// collection prefix), and HTTP(S) Server collection forms. The expanded form
/// has no trailing slash. Embedded credentials, queries, and fragments are
/// rejected as library constraints for an organization base URL.
///
/// Cloud organization names and Server collection names have different platform
/// rules and are represented separately. Server collection URLs encode their
/// exact name as a path segment, including spaces and Unicode.
/// Parsing also accepts a cloud organization name such as `abc123`, expanding it
/// to `https://dev.azure.com/abc123` without consulting configured defaults.
///
/// See Microsoft's [organization URL forms](https://learn.microsoft.com/en-us/azure/devops/extend/develop/work-with-urls?view=azure-devops)
/// and [REST instance/collection URL structure](https://learn.microsoft.com/en-us/rest/api/azure/devops/?view=azure-devops-rest-7.1#components-of-a-rest-api-requestresponse-pair).
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsOrganizationUrl {
    // Private to preserve the validated base URL and its agreement with `name`.
    base_url: Url,
    // Changing the scope alone could contradict the cloud host or legacy name.
    name: AzureDevOpsOrganizationUrlName,
}

impl AzureDevOpsOrganizationUrl {
    /// Construct a validated instance prefix and organization/collection name.
    ///
    /// Prefer [`Self::try_new_server_collection`] when the name follows Server
    /// collection rules rather than the narrower cloud organization vocabulary.
    pub fn new(
        base_url: impl Into<CompactString>,
        organization_name: impl Into<AzureDevOpsOrganizationName>,
    ) -> Result<Self> {
        Self::try_new(base_url.into(), organization_name.into())
    }

    pub fn try_new<B, N>(base_url: B, organization_name: N) -> Result<Self>
    where
        B: TryInto<CompactString>,
        B::Error: Into<eyre::Error>,
        N: TryInto<AzureDevOpsOrganizationName>,
        N::Error: Into<eyre::Error>,
    {
        let base_url = base_url
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert base_url")?;
        let organization_name = organization_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert organization_name")?;
        let mut url = Url::parse(&base_url).wrap_err("Invalid Azure DevOps base URL")?;
        validate_base_url(&url)?;
        let path = url.path().trim_end_matches('/').to_owned();
        url.set_path(&path);
        let cloud_host = url
            .host_str()
            .is_some_and(|host| host == "dev.azure.com" || host.ends_with(".visualstudio.com"));
        if cloud_host {
            if url.host_str() == Some("dev.azure.com") {
                ensure!(
                    url.path().trim_matches('/').is_empty(),
                    "The modern organization base must not already contain a scope path"
                );
            } else {
                let host_name = url
                    .host_str()
                    .and_then(|host| host.strip_suffix(".visualstudio.com"))
                    .expect("validated legacy cloud host");
                AzureDevOpsOrganizationName::try_new(host_name)?;
                ensure!(
                    host_name.eq_ignore_ascii_case(organization_name.as_ref()),
                    "Legacy organization host and organization name must agree"
                );
                ensure!(
                    url.path_segments()
                        .expect("HTTP(S) URLs have hierarchical paths")
                        .filter(|segment| !segment.is_empty())
                        .count()
                        <= 1,
                    "Legacy organization bases may contain only a collection prefix"
                );
            }
            Ok(Self {
                base_url: url,
                name: AzureDevOpsOrganizationUrlName::Organization(organization_name),
            })
        } else {
            Ok(Self {
                base_url: url,
                name: AzureDevOpsOrganizationUrlName::ServerCollection(
                    AzureDevOpsProjectCollectionName::try_new(organization_name.to_string())?,
                ),
            })
        }
    }

    /// Creates a new Azure DevOps organization URL with the standard dev.azure.com base
    pub fn new_dev_azure_com(organization_name: impl Into<AzureDevOpsOrganizationName>) -> Self {
        Self {
            base_url: Url::parse("https://dev.azure.com")
                .expect("the standard organization base is a valid URL"),
            name: AzureDevOpsOrganizationUrlName::Organization(organization_name.into()),
        }
    }

    pub fn try_new_dev_azure_com<N>(organization_name: N) -> Result<Self>
    where
        N: TryInto<AzureDevOpsOrganizationName>,
        N::Error: Into<eyre::Error>,
    {
        let organization_name = organization_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert organization_name")?;
        Ok(Self::new_dev_azure_com(organization_name))
    }

    /// Creates a legacy visualstudio.com URL with a canonical host spelling.
    /// The typed organization name retains its supplied spelling.
    pub fn new_visual_studio_com(
        organization_name: impl Into<AzureDevOpsOrganizationName>,
    ) -> Self {
        let org_name = organization_name.into();
        let base_url = format!("https://{}.visualstudio.com", org_name.as_ref());
        Self {
            base_url: Url::parse(&base_url)
                .expect("a validated organization name forms a valid legacy URL"),
            name: AzureDevOpsOrganizationUrlName::Organization(org_name),
        }
    }

    pub fn try_new_visual_studio_com<N>(organization_name: N) -> Result<Self>
    where
        N: TryInto<AzureDevOpsOrganizationName>,
        N::Error: Into<eyre::Error>,
    {
        let organization_name = organization_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert organization_name")?;
        Ok(Self::new_visual_studio_com(organization_name))
    }

    /// Construct a Server collection URL from its instance prefix and exact name.
    pub fn try_new_server_collection(
        base_url: impl Into<CompactString>,
        collection_name: AzureDevOpsProjectCollectionName,
    ) -> Result<Self> {
        let base_url = base_url.into();
        let mut url = Url::parse(&base_url).wrap_err("Invalid Azure DevOps Server instance URL")?;
        validate_base_url(&url)?;
        ensure!(
            url.host_str() != Some("dev.azure.com")
                && !url
                    .host_str()
                    .is_some_and(|host| host.ends_with(".visualstudio.com")),
            "Server collections require a Server instance URL rather than a cloud organization host"
        );
        let path = url.path().trim_end_matches('/').to_owned();
        url.set_path(&path);
        Ok(Self {
            base_url: url,
            name: AzureDevOpsOrganizationUrlName::ServerCollection(collection_name),
        })
    }

    /// Return the instance prefix without appending the organization/collection.
    pub fn base_url(&self) -> &str {
        self.base_url.as_str().trim_end_matches('/')
    }

    /// Return a cloud organization name, rejecting Server collection scopes.
    pub fn organization_name(&self) -> Result<&AzureDevOpsOrganizationName> {
        match &self.name {
            AzureDevOpsOrganizationUrlName::Organization(organization_name) => {
                Ok(organization_name)
            }
            AzureDevOpsOrganizationUrlName::ServerCollection(_) => {
                bail!("Server collections do not have a cloud organization name")
            }
        }
    }

    pub fn collection_name(&self) -> Option<&AzureDevOpsProjectCollectionName> {
        match &self.name {
            AzureDevOpsOrganizationUrlName::Organization(_) => None,
            AzureDevOpsOrganizationUrlName::ServerCollection(collection_name) => {
                Some(collection_name)
            }
        }
    }

    /// Return the actual scope name for display or cache namespacing.
    pub fn name(&self) -> &str {
        match &self.name {
            AzureDevOpsOrganizationUrlName::Organization(organization_name) => {
                organization_name.as_ref()
            }
            AzureDevOpsOrganizationUrlName::ServerCollection(collection_name) => {
                collection_name.as_str()
            }
        }
    }

    pub fn expanded_form(&self) -> String {
        match &self.name {
            AzureDevOpsOrganizationUrlName::Organization(organization_name) => {
                if self.is_visual_studio_com_format() {
                    // The legacy host identifies the organization; preserve its
                    // collection prefix instead of appending the name again.
                    self.base_url().to_owned()
                } else {
                    let mut url = self.base_url.clone();
                    url.path_segments_mut()
                        .expect("organization bases have hierarchical HTTP(S) paths")
                        .pop_if_empty()
                        .push(organization_name.as_ref());
                    url.into()
                }
            }
            AzureDevOpsOrganizationUrlName::ServerCollection(collection_name) => {
                let mut url = self.base_url.clone();
                url.path_segments_mut()
                    .expect("Server instance has hierarchical HTTP(S) paths")
                    .pop_if_empty()
                    .push(collection_name.as_str());
                url.into()
            }
        }
    }

    /// Build a REST endpoint for a cloud API area's service host, encoding query
    /// parameters with [`Url`]. Legacy organization URLs retain their host and
    /// collection prefix. Server collection scopes do not have a cloud
    /// organization name and return an error.
    ///
    /// See Microsoft's [Azure DevOps service URL forms](https://learn.microsoft.com/en-us/azure/devops/extend/develop/work-with-urls?view=azure-devops).
    #[track_caller]
    pub fn api_url(&self, host: &str, path: &str, query: &[(&str, &str)]) -> Result<Url> {
        let base = if self.is_visual_studio_com_format() {
            self.expanded_form()
        } else {
            format!("https://{host}/{}", self.organization_name()?.as_ref())
        };
        let mut url = Url::parse(&format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        ))?;
        {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in query {
                pairs.append_pair(name, value);
            }
        }
        Ok(url)
    }

    pub fn is_dev_azure_com_format(&self) -> bool {
        matches!(self.name, AzureDevOpsOrganizationUrlName::Organization(_))
            && self.base_url.host_str() == Some("dev.azure.com")
    }

    pub fn is_visual_studio_com_format(&self) -> bool {
        matches!(self.name, AzureDevOpsOrganizationUrlName::Organization(_))
            && self
                .base_url
                .host_str()
                .and_then(|host| host.strip_suffix(".visualstudio.com"))
                .is_some_and(|organization| !organization.is_empty())
    }
}

impl std::fmt::Display for AzureDevOpsOrganizationUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expanded_form())
    }
}

impl FromStr for AzureDevOpsOrganizationUrl {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // Treat a bare value as an organization name and use the modern
        // Azure DevOps host. Full URLs are handled below so callers can
        // explicitly select the legacy visualstudio.com form.
        if !value.contains("://") {
            return Self::try_new_dev_azure_com(value);
        }

        let mut url = Url::parse(value).wrap_err("Invalid Azure DevOps organization URL")?;
        validate_base_url(&url)?;
        let host = url.host_str().expect("validated URL host").to_owned();
        let path = url.path().trim_end_matches('/').to_owned();
        url.set_path(&path);
        ensure!(
            !url.path().contains("//"),
            "Azure DevOps organization URLs must not contain empty path segments"
        );
        let segments: Vec<_> = url
            .path_segments()
            .expect("HTTP(S) URLs have hierarchical paths")
            .filter(|segment| !segment.is_empty())
            .collect();

        if let Some(organization) = host.strip_suffix(".visualstudio.com") {
            ensure!(
                segments.len() <= 1,
                "Legacy organization URLs may contain only a collection prefix"
            );
            let organization_name = AzureDevOpsOrganizationName::try_new(organization)
                .wrap_err("Invalid organization name in legacy URL")?;
            return Ok(Self {
                base_url: url,
                name: AzureDevOpsOrganizationUrlName::Organization(organization_name),
            });
        }

        if host == "dev.azure.com" {
            ensure!(
                segments.len() == 1,
                "Modern organization URLs require exactly one organization path segment"
            );
        }
        let Some(name) = segments.last() else {
            bail!("Organization or Server collection name is missing from URL");
        };
        let name = decode_name_segment(name)?;
        url.path_segments_mut()
            .expect("HTTP(S) URLs have hierarchical paths")
            .pop();
        let path = url.path().trim_end_matches('/').to_owned();
        url.set_path(&path);
        if host == "dev.azure.com" {
            Ok(Self {
                base_url: url,
                name: AzureDevOpsOrganizationUrlName::Organization(
                    AzureDevOpsOrganizationName::try_new(name)
                        .wrap_err("Invalid cloud organization name")?,
                ),
            })
        } else {
            Ok(Self {
                base_url: url,
                name: AzureDevOpsOrganizationUrlName::ServerCollection(
                    AzureDevOpsProjectCollectionName::try_new(name)
                        .wrap_err("Invalid Server project collection name")?,
                ),
            })
        }
    }
}

fn validate_base_url(url: &Url) -> Result<()> {
    ensure!(
        matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
        "Azure DevOps organization URLs must be absolute HTTP(S) URLs"
    );
    ensure!(
        url.username().is_empty() && url.password().is_none(),
        "Azure DevOps organization URLs must not contain credentials"
    );
    ensure!(
        url.query().is_none() && url.fragment().is_none(),
        "Azure DevOps organization URLs must not contain queries or fragments"
    );
    Ok(())
}

/// Decode the final name segment without treating a literal `+` as a space.
fn decode_name_segment(segment: &str) -> Result<String> {
    let mut decoded = Vec::with_capacity(segment.len());
    let mut bytes = segment.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = bytes
                .next()
                .and_then(|byte| (byte as char).to_digit(16))
                .ok_or_else(|| eyre::eyre!("Invalid percent escape in organization name"))?;
            let low = bytes
                .next()
                .and_then(|byte| (byte as char).to_digit(16))
                .ok_or_else(|| eyre::eyre!("Invalid percent escape in organization name"))?;
            decoded.push(((high << 4) | low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    Ok(String::from_utf8(decoded)?)
}

impl<'a> Arbitrary<'a> for AzureDevOpsOrganizationUrl {
    fn arbitrary(unstructured: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let legacy = bool::arbitrary(unstructured)?;
        let organization_name = AzureDevOpsOrganizationName::arbitrary(unstructured)?;
        Ok(if legacy {
            // URL parsers normalize DNS host case. Generate the canonical host
            // spelling so nested URL values retain the same typed identity.
            let organization_name =
                AzureDevOpsOrganizationName::try_new(organization_name.to_ascii_lowercase())
                    .map_err(|_| arbitrary::Error::IncorrectFormat)?;
            Self::new_visual_studio_com(organization_name)
        } else {
            Self::new_dev_azure_com(organization_name)
        })
    }
}

impl TryFrom<&str> for AzureDevOpsOrganizationUrl {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for AzureDevOpsOrganizationUrl {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<&AzureDevOpsOrganizationUrl> for String {
    fn from(value: &AzureDevOpsOrganizationUrl) -> Self {
        value.expanded_form()
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsOrganizationUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsOrganizationUrl);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_azure_com_format() -> Result<()> {
        let url = AzureDevOpsOrganizationUrl::try_new_dev_azure_com("myorg")?;
        assert_eq!(url.expanded_form(), "https://dev.azure.com/myorg");
        assert!(url.is_dev_azure_com_format());
        assert!(!url.is_visual_studio_com_format());
        Ok(())
    }

    #[test]
    fn test_visual_studio_com_format() -> Result<()> {
        let url = AzureDevOpsOrganizationUrl::try_new_visual_studio_com("myorg")?;
        assert_eq!(url.expanded_form(), "https://myorg.visualstudio.com");
        assert!(!url.is_dev_azure_com_format());
        assert!(url.is_visual_studio_com_format());
        Ok(())
    }

    #[test]
    fn test_parse_dev_azure_com() -> Result<()> {
        let url = "https://dev.azure.com/myorg".parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(url.base_url(), "https://dev.azure.com");
        assert_eq!(url.organization_name()?.as_ref(), "myorg");
        assert_eq!(url.expanded_form(), "https://dev.azure.com/myorg");
        Ok(())
    }

    #[test]
    fn organization_name_shorthand_is_normalized() -> Result<()> {
        let shorthand = "synthetic".parse::<AzureDevOpsOrganizationUrl>()?;
        let explicit = "https://dev.azure.com/synthetic".parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(shorthand, explicit);
        assert_eq!(shorthand.to_string(), "https://dev.azure.com/synthetic");
        assert!(
            "synthetic/project"
                .parse::<AzureDevOpsOrganizationUrl>()
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn test_parse_visual_studio_com() -> Result<()> {
        let url = "https://myorg.visualstudio.com".parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(url.base_url(), "https://myorg.visualstudio.com");
        assert_eq!(url.organization_name()?.as_ref(), "myorg");
        assert_eq!(url.expanded_form(), "https://myorg.visualstudio.com");
        Ok(())
    }

    #[test]
    fn test_roundtrip_serialization() -> Result<()> {
        let original_dev = AzureDevOpsOrganizationUrl::try_new_dev_azure_com("test-org")?;
        let serialized = facet_json::to_string(&original_dev)?;
        let deserialized: AzureDevOpsOrganizationUrl = facet_json::from_str(&serialized)?;
        assert_eq!(original_dev, deserialized);

        let original_vs = AzureDevOpsOrganizationUrl::try_new_visual_studio_com("test-org")?;
        let serialized = facet_json::to_string(&original_vs)?;
        let deserialized: AzureDevOpsOrganizationUrl = facet_json::from_str(&serialized)?;
        assert_eq!(original_vs, deserialized);
        Ok(())
    }

    #[test]
    fn test_invalid_urls() {
        assert!(AzureDevOpsOrganizationUrl::from_str("https://example.com").is_err());
        assert!(AzureDevOpsOrganizationUrl::from_str("https://dev.azure.com/").is_err());
        assert!(AzureDevOpsOrganizationUrl::from_str("https://.visualstudio.com").is_err());
        assert!(AzureDevOpsOrganizationUrl::from_str("-not-a-valid-organization").is_err());
    }

    #[test]
    fn legacy_collection_prefix_keeps_the_host_organization_identity() -> Result<()> {
        let url = "https://myorg.visualstudio.com/DefaultCollection/"
            .parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(url.organization_name()?.as_ref(), "myorg");
        assert!(url.is_visual_studio_com_format());
        assert_eq!(
            url.expanded_form(),
            "https://myorg.visualstudio.com/DefaultCollection"
        );
        Ok(())
    }

    #[test]
    fn server_collection_urls_preserve_the_instance_prefix_and_port() -> Result<()> {
        for (value, base, name) in [
            (
                "http://server.example.invalid:8080/tfs/DefaultCollection",
                "http://server.example.invalid:8080/tfs",
                "DefaultCollection",
            ),
            (
                "https://server.example.invalid/Collection-1",
                "https://server.example.invalid",
                "Collection-1",
            ),
            (
                "http://server.example.invalid:8080/tfs/My%20Collection",
                "http://server.example.invalid:8080/tfs",
                "My Collection",
            ),
            (
                "https://server.example.invalid/Collection_1",
                "https://server.example.invalid",
                "Collection_1",
            ),
            (
                "http://server.example.invalid:8080/tfs%20instance/My%20Collection",
                "http://server.example.invalid:8080/tfs%20instance",
                "My Collection",
            ),
            (
                "https://server.example.invalid/tfs/%E9%9B%86%E5%90%88",
                "https://server.example.invalid/tfs",
                "集合",
            ),
        ] {
            let url = value.parse::<AzureDevOpsOrganizationUrl>()?;
            assert_eq!(url.base_url(), base);
            assert_eq!(url.collection_name().unwrap().as_str(), name);
            assert_eq!(url.name(), name);
            assert!(url.organization_name().is_err());
            assert_eq!(url.expanded_form(), value);
            assert!(!url.is_dev_azure_com_format());
            assert!(!url.is_visual_studio_com_format());
        }
        let name = AzureDevOpsProjectCollectionName::try_new("a".repeat(64))?;
        let url = AzureDevOpsOrganizationUrl::try_new_server_collection(
            "https://server.example.invalid/tfs",
            name.clone(),
        )?;
        assert_eq!(url.collection_name(), Some(&name));
        assert_eq!(
            AzureDevOpsOrganizationUrl::try_new_server_collection(
                "https://server.example.invalid/tfs/",
                name,
            )?,
            url
        );
        assert_eq!(
            url.expanded_form().parse::<AzureDevOpsOrganizationUrl>()?,
            url
        );
        Ok(())
    }

    #[test]
    fn parsing_enforces_base_url_boundaries_and_decodes_the_name() -> Result<()> {
        for value in [
            "ftp://server.example.invalid/DefaultCollection",
            "https://user:password@server.example.invalid/DefaultCollection",
            "https://dev.azure.com/myorg?project=sample",
            "https://dev.azure.com/myorg#fragment",
            "https://dev.azure.com/myorg/project",
        ] {
            assert!(value.parse::<AzureDevOpsOrganizationUrl>().is_err());
        }
        let encoded = "https://dev.azure.com/%6D%79org".parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(encoded.organization_name()?.as_ref(), "myorg");
        assert_eq!(encoded.expanded_form(), "https://dev.azure.com/myorg");
        let name = AzureDevOpsOrganizationName::try_new("myorg")?;
        for base in [
            "invalid base",
            "https://user:password@dev.azure.com",
            "https://dev.azure.com?query=1",
            "https://dev.azure.com/myorg",
            "https://otherorg.visualstudio.com",
        ] {
            assert!(AzureDevOpsOrganizationUrl::new(base, name.clone()).is_err());
        }
        Ok(())
    }

    #[test]
    fn arbitrary_urls_have_valid_offline_names_and_roundtrip() -> Result<()> {
        for seed in 0..32_u8 {
            let bytes: Vec<_> = (0..256)
                .map(|offset| seed.wrapping_add(offset as u8))
                .collect();
            let mut unstructured = arbitrary::Unstructured::new(&bytes);
            let url = AzureDevOpsOrganizationUrl::arbitrary(&mut unstructured)?;
            assert!(Url::parse(&url.expanded_form()).is_ok());
            assert_eq!(
                url.expanded_form().parse::<AzureDevOpsOrganizationUrl>()?,
                url
            );
        }
        Ok(())
    }

    #[test]
    fn test_display() -> Result<()> {
        let url_dev = AzureDevOpsOrganizationUrl::try_new_dev_azure_com("myorg")?;
        assert_eq!(url_dev.to_string(), "https://dev.azure.com/myorg");

        let url_vs = AzureDevOpsOrganizationUrl::try_new_visual_studio_com("myorg")?;
        assert_eq!(url_vs.to_string(), "https://myorg.visualstudio.com");
        Ok(())
    }

    #[test]
    fn test_with_trailing_slash() -> Result<()> {
        let url = "https://dev.azure.com/myorg/".parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(url.organization_name()?.as_ref(), "myorg");
        assert_eq!(url.expanded_form(), "https://dev.azure.com/myorg");
        Ok(())
    }
}
