use crate::prelude::AzureDevOpsOrganizationName;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct AzureDevOpsOrganizationUrl {
    pub base_url: CompactString,
    pub organization_name: AzureDevOpsOrganizationName,
}

impl AzureDevOpsOrganizationUrl {
    pub fn new(
        base_url: impl Into<CompactString>,
        organization_name: impl Into<AzureDevOpsOrganizationName>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            organization_name: organization_name.into(),
        }
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
        Ok(Self {
            base_url,
            organization_name,
        })
    }

    /// Creates a new Azure DevOps organization URL with the standard dev.azure.com base
    pub fn new_dev_azure_com(organization_name: impl Into<AzureDevOpsOrganizationName>) -> Self {
        Self::new("https://dev.azure.com", organization_name)
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

    /// Creates a new Azure DevOps organization URL with the legacy visualstudio.com base
    pub fn new_visual_studio_com(
        organization_name: impl Into<AzureDevOpsOrganizationName>,
    ) -> Self {
        let org_name = organization_name.into();
        let base_url = format!("https://{}.visualstudio.com", org_name.as_ref());
        Self {
            base_url: base_url.into(),
            organization_name: org_name,
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

    pub fn expanded_form(&self) -> String {
        if self.base_url.ends_with(".visualstudio.com") {
            // For legacy format, org name is already in the base URL
            self.base_url.to_string()
        } else {
            // For modern format, append org name to base URL
            format!(
                "{}/{}",
                self.base_url.trim_end_matches('/'),
                self.organization_name
            )
        }
    }

    pub fn is_dev_azure_com_format(&self) -> bool {
        self.base_url.starts_with("https://dev.azure.com")
    }

    pub fn is_visual_studio_com_format(&self) -> bool {
        self.base_url.ends_with(".visualstudio.com")
    }
}

impl std::fmt::Display for AzureDevOpsOrganizationUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.expanded_form())
    }
}

impl FromStr for AzureDevOpsOrganizationUrl {
    type Err = eyre::Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        // Handle dev.azure.com format: https://dev.azure.com/{organization}
        if let Some(org_part) = url.strip_prefix("https://dev.azure.com/") {
            let org_name = org_part.trim_end_matches('/');
            if org_name.is_empty() {
                bail!("Organization name is missing from URL: {}", url);
            }

            let organization_name = AzureDevOpsOrganizationName::try_new(org_name).context(
                format!("Invalid organization name '{}' in URL: {}", org_name, url),
            )?;

            return Ok(Self::new("https://dev.azure.com", organization_name));
        }

        // Handle visualstudio.com format: https://{organization}.visualstudio.com
        if url.starts_with("https://") && url.ends_with(".visualstudio.com") {
            let without_protocol = url.strip_prefix("https://").unwrap();
            let org_name = without_protocol.strip_suffix(".visualstudio.com").unwrap();

            if org_name.is_empty() {
                bail!("Organization name is missing from URL: {}", url);
            }

            let organization_name = AzureDevOpsOrganizationName::try_new(org_name).context(
                format!("Invalid organization name '{}' in URL: {}", org_name, url),
            )?;

            return Ok(Self::new_visual_studio_com(organization_name));
        }

        bail!(
            "URL '{}' does not match expected Azure DevOps organization URL format",
            url
        );
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

impl Serialize for AzureDevOpsOrganizationUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for AzureDevOpsOrganizationUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url = String::deserialize(deserializer)?;
        Self::from_str(&url).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}

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
        assert_eq!(url.base_url, "https://dev.azure.com");
        assert_eq!(url.organization_name.as_ref(), "myorg");
        assert_eq!(url.expanded_form(), "https://dev.azure.com/myorg");
        Ok(())
    }

    #[test]
    fn test_parse_visual_studio_com() -> Result<()> {
        let url = "https://myorg.visualstudio.com".parse::<AzureDevOpsOrganizationUrl>()?;
        assert_eq!(url.base_url, "https://myorg.visualstudio.com");
        assert_eq!(url.organization_name.as_ref(), "myorg");
        assert_eq!(url.expanded_form(), "https://myorg.visualstudio.com");
        Ok(())
    }

    #[test]
    fn test_roundtrip_serialization() -> Result<()> {
        let original_dev = AzureDevOpsOrganizationUrl::try_new_dev_azure_com("test-org")?;
        let serialized = serde_json::to_string(&original_dev)?;
        println!("Serialized: {}", serialized);
        let deserialized: AzureDevOpsOrganizationUrl = serde_json::from_str(&serialized)?;
        assert_eq!(original_dev, deserialized);

        let original_vs = AzureDevOpsOrganizationUrl::try_new_visual_studio_com("test-org")?;
        let serialized = serde_json::to_string(&original_vs)?;
        println!("Serialized: {}", serialized);
        let deserialized: AzureDevOpsOrganizationUrl = serde_json::from_str(&serialized)?;
        assert_eq!(original_vs, deserialized);
        Ok(())
    }

    #[test]
    fn test_invalid_urls() {
        assert!(AzureDevOpsOrganizationUrl::from_str("https://example.com").is_err());
        assert!(AzureDevOpsOrganizationUrl::from_str("https://dev.azure.com/").is_err());
        assert!(AzureDevOpsOrganizationUrl::from_str("https://.visualstudio.com").is_err());
        assert!(AzureDevOpsOrganizationUrl::from_str("not-a-url").is_err());
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
        assert_eq!(url.organization_name.as_ref(), "myorg");
        assert_eq!(url.expanded_form(), "https://dev.azure.com/myorg");
        Ok(())
    }
}
