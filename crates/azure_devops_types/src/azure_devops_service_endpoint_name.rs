use serde::de::Error;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use std::ops::Deref;
use std::str::FromStr;
use validator::Validate;

#[derive(Debug, Eq, PartialEq, Clone, Validate, Hash)]
pub struct AzureDevOpsServiceEndpointName {
    #[validate(length(min = 1))]
    inner: CompactString,
}

impl Deref for AzureDevOpsServiceEndpointName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for AzureDevOpsServiceEndpointName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

impl AzureDevOpsServiceEndpointName {
    pub fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let org = Self { inner: name.into() };
        org.validate()?;
        Ok(org)
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
        &self.inner
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
            
        let name = AzureDevOpsServiceEndpointName { inner: chars?.into() };
        if name.validate().is_ok() {
            Ok(name)
        } else {
            Err(arbitrary::Error::IncorrectFormat)
        }
    }
}

impl serde::Serialize for AzureDevOpsServiceEndpointName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureDevOpsServiceEndpointName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}


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
                println!("Generated: {:?} - \"{}\"", name, name);
                let validation = name.validate();
                assert!(
                    validation.is_ok(),
                    "Arbitrary produced invalid: {:?} - {:?}",
                    &name.inner,
                    validation
                );
            }
        }
    }
}
