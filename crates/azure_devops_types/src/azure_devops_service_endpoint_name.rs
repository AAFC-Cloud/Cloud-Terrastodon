use arbitrary::Arbitrary;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone, Hash, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsServiceEndpointName(CompactString);

impl Deref for AzureDevOpsServiceEndpointName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for AzureDevOpsServiceEndpointName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AzureDevOpsServiceEndpointName {
    pub fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        if inner.is_empty() {
            eyre::bail!("Service endpoint name cannot be empty");
        }
        Ok(Self(inner))
    }
}

impl FromStr for AzureDevOpsServiceEndpointName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl AsRef<str> for AzureDevOpsServiceEndpointName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for AzureDevOpsServiceEndpointName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for AzureDevOpsServiceEndpointName {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsServiceEndpointName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Generate length between 1 and 64 (reasonable for service endpoint names)
        let len = u.int_in_range(1..=64)?;

        // Generate only printable ASCII characters (32-126)
        let chars: Result<String, _> = (0..len)
            .map(|_| {
                let byte = u.int_in_range(32u8..=126u8)?;
                Ok(byte as char)
            })
            .collect();

        let name = AzureDevOpsServiceEndpointName(chars?.into());
        // Since we generate length 1..=64, it's guaranteed to be non-empty
        Ok(name)
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsServiceEndpointName);

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    fn test_valid_examples() {
        assert!(AzureDevOpsServiceEndpointName::try_new("bruh").is_ok());
        assert!(AzureDevOpsServiceEndpointName::try_new("org-123").is_ok());
        assert!(AzureDevOpsServiceEndpointName::try_new("Test-serviceendpoint-2024").is_ok());
        assert!(AzureDevOpsServiceEndpointName::try_new("a").is_ok());
        assert!(AzureDevOpsServiceEndpointName::try_new("1").is_ok());
    }

    #[test]
    fn roundtrip_display_and_fromstr() {
        let name = "myserviceendpoint-123";
        let org = AzureDevOpsServiceEndpointName::from_str(name).unwrap();
        assert_eq!(org.to_string(), name);
        assert_eq!(org.as_ref(), name);
    }

    #[test]
    fn arbitrary_generates_valid_names() {
        for _ in 0..100 {
            let raw: Vec<u8> = (0..128).map(|_| rand::random::<u8>()).collect();
            let mut u = Unstructured::new(&raw);
            if let Ok(name) = AzureDevOpsServiceEndpointName::arbitrary(&mut u) {
                // Since Arbitrary generates valid names, just check it's not empty
                assert!(!name.0.is_empty(), "Arbitrary produced empty name");
            }
        }
    }
}

