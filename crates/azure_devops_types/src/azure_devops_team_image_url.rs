use arbitrary::Arbitrary;
use eyre::Result;
use eyre::ensure;
use std::str::FromStr;
use url::Url;

/// The URL of a team identity image returned by Azure DevOps.
///
/// See `defaultTeamImageUrl` in Microsoft's [TeamProjectReference 7.1 schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/core/projects/list?view=azure-devops-rest-7.1#teamprojectreference).
/// The documented image URL has no fixed route or host contract, so this type
/// preserves its path, query, and fragment without inferring an identity.
/// Absolute HTTP(S) URLs without embedded credentials are library safety
/// constraints, rather than guarantees stated by that schema.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(proxy = String)]
pub struct AzureDevOpsTeamImageUrl(Url);

impl AzureDevOpsTeamImageUrl {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        Url::parse(value.as_ref())?.try_into()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

impl FromStr for AzureDevOpsTeamImageUrl {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsTeamImageUrl {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for AzureDevOpsTeamImageUrl {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self> {
        Self::try_new(value)
    }
}

impl TryFrom<Url> for AzureDevOpsTeamImageUrl {
    type Error = eyre::Report;

    fn try_from(value: Url) -> Result<Self> {
        ensure!(
            matches!(value.scheme(), "http" | "https") && value.host_str().is_some(),
            "Azure DevOps image URLs must be absolute HTTP(S) URLs"
        );
        ensure!(
            value.username().is_empty() && value.password().is_none(),
            "Azure DevOps image URLs must not contain credentials"
        );
        Ok(Self(value))
    }
}

impl AsRef<str> for AzureDevOpsTeamImageUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<Url> for AzureDevOpsTeamImageUrl {
    fn as_ref(&self) -> &Url {
        self.as_url()
    }
}

impl From<&AzureDevOpsTeamImageUrl> for String {
    fn from(value: &AzureDevOpsTeamImageUrl) -> Self {
        value.as_str().to_owned()
    }
}

impl From<AzureDevOpsTeamImageUrl> for Url {
    fn from(value: AzureDevOpsTeamImageUrl) -> Self {
        value.into_url()
    }
}

impl std::fmt::Display for AzureDevOpsTeamImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsTeamImageUrl {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let image_id = u64::arbitrary(u)?;
        format!("https://images.invalid/team/{image_id}?size=small")
            .parse()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamImageUrl);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamImageUrl);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_opaque_image_urls() -> Result<()> {
        for text in [
            "https://images.invalid/opaque/avatar?identity=synthetic&size=small#preview",
            "https://synthetic.visualstudio.com/DefaultCollection/_api/_common/identityImage?id=synthetic",
            "http://devops.invalid:8080/custom/image?version=2",
        ] {
            let url: AzureDevOpsTeamImageUrl = text.parse()?;
            assert_eq!(url.as_str(), text);
            assert_eq!(url.clone().into_url(), Url::parse(text)?);
        }
        Ok(())
    }

    #[test]
    fn enforces_http_safety_during_parsing() -> Result<()> {
        for text in [
            "relative/team/image",
            "ftp://images.invalid/avatar",
            "data:image/png;base64,synthetic",
            "https://username@images.invalid/avatar",
            "https://username:password@images.invalid/avatar",
        ] {
            assert!(text.parse::<AzureDevOpsTeamImageUrl>().is_err());
        }
        Ok(())
    }

    #[test]
    fn arbitrary_images_are_valid_and_optional_values_need_no_field_overrides() -> Result<()> {
        for seed in 0..32 {
            let bytes = [seed; 128];
            let mut u = arbitrary::Unstructured::new(&bytes);
            let url = AzureDevOpsTeamImageUrl::arbitrary(&mut u)?;
            assert_eq!(url.as_url().host_str(), Some("images.invalid"));
            assert_eq!(url.as_str().parse::<AzureDevOpsTeamImageUrl>()?, url);
            let optional: Option<AzureDevOpsTeamImageUrl> = u.arbitrary()?;
            if let Some(url) = optional {
                assert_eq!(url.as_str().parse::<AzureDevOpsTeamImageUrl>()?, url);
            }
        }
        Ok(())
    }
}
