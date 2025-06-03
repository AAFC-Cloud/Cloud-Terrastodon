use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use serde::de::Error;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;
use validator::Validate;
use validator::ValidationError;

use crate::slug::Slug;

const CONTAINER_REGISTRY_NAMING_RULES_URL: &str = "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftcontainerregistry";
#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate, PartialOrd, Ord)]
pub struct ContainerRegistryRepositoryName {
    // #[validate(length(min = 5, max = 50), custom(function = "validate_alphanumeric"))]
    pub inner: CompactString,
}
impl Slug for ContainerRegistryRepositoryName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let rtn = Self { inner: name.into() };
        rtn.validate()?;
        Ok(rtn)
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        self.validate()?;
        Ok(())
    }
}
// fn validate_alphanumeric(value: &CompactString) -> Result<(), ValidationError> {
//     for (i, char) in value.chars().enumerate() {
//         if !char.is_ascii_alphanumeric() {
//             return Err(
//                 ValidationError::new(CONTAINER_REGISTRY_NAMING_RULES_URL).with_message(
//                     format!("Char {char} as position {i} in {value:?} must be alphanumeric").into(),
//                 ),
//             );
//         }
//     }
//     Ok(())
// }

impl FromStr for ContainerRegistryRepositoryName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ContainerRegistryRepositoryName::try_new(s)
    }
}
impl TryFrom<&str> for ContainerRegistryRepositoryName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ContainerRegistryRepositoryName::try_new(value)
    }
}
impl std::fmt::Display for ContainerRegistryRepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for ContainerRegistryRepositoryName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ContainerRegistryRepositoryName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <CompactString as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}
impl Deref for ContainerRegistryRepositoryName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for ContainerRegistryRepositoryName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl TryFrom<CompactString> for ContainerRegistryRepositoryName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}
impl From<ContainerRegistryRepositoryName> for CompactString {
    fn from(value: ContainerRegistryRepositoryName) -> Self {
        value.inner
    }
}

// impl<'a> Arbitrary<'a> for ContainerRegistryRepositoryName {
//     fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
//         // Get length in 3-24
//         let len = u.int_in_range(3..=24)?;
//         // Use only [a-z]
//         let choices = ('a'..='Z').chain('1'..='9').collect::<Vec<_>>();
//         let name: String = (0..len)
//             .map(|_| {
//                 // Safe since 'a'..'z' is always valid
//                 let c = u.choose(&choices)?;
//                 Ok(*c)
//             })
//             .collect::<Result<String, _>>()?;
//         ContainerRegistryRepositoryName::try_new(CompactString::from(name))
//             .map_err(|_| arbitrary::Error::IncorrectFormat)
//     }
// }

#[cfg(test)]
mod test {
    use crate::prelude::ContainerRegistryRepositoryName;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;
    use rand::Rng;
    use validator::Validate;

    #[test]
    pub fn validation() -> eyre::Result<()> {
        assert!(ContainerRegistryRepositoryName::try_new("bruh").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("-").is_err());
        assert!(ContainerRegistryRepositoryName::try_new("a-b-c").is_err());
        assert!(ContainerRegistryRepositoryName::try_new("hi+hi").is_err());
        assert!(ContainerRegistryRepositoryName::try_new("").is_err());
        assert!(ContainerRegistryRepositoryName::try_new("a").is_err());
        assert!(ContainerRegistryRepositoryName::try_new("aa").is_err());
        assert!(ContainerRegistryRepositoryName::try_new("JEOFF").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("caPital").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("aaaa").is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a".repeat(23)).is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a".repeat(24)).is_ok());
        assert!(ContainerRegistryRepositoryName::try_new("a".repeat(25)).is_err());
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn preview_failure() -> eyre::Result<()> {
        ContainerRegistryRepositoryName::try_new("abc123---B321")?;
        Ok(())
    }

    // #[test]
    // pub fn fuzz() -> eyre::Result<()> { // TODO: figure out Error: The raw data is not of the correct format to construct this type
    //     let mut found_uppercase = false;
    //     for _ in 0..100 {
    //         let mut raw = [0u8; 64];
    //         rand::thread_rng().fill(&mut raw);
    //         let mut un = Unstructured::new(&raw);
    //         let name = ContainerRegistryRepositoryName::arbitrary(&mut un)?;
    //         assert!(name.validate().is_ok());
    //         if name.chars().any(|c| c.is_ascii_uppercase()) {
    //             found_uppercase = true;
    //         }
    //         println!("{name}");
    //     }
    //     assert!(
    //         found_uppercase,
    //         "At least one generated name should contain an uppercase character"
    //     );
    //     Ok(())
    // }
}
