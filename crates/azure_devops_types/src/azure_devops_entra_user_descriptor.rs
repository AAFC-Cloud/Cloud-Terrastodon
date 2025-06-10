use serde::de::Error;
use std::ops::Deref;
use std::ops::DerefMut;
use validator::Validate;
use validator::ValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Validate)]
pub struct AzureDevOpsEntraUserDescriptor {
    #[validate(custom(function = "validate_has_prefix"))]
    inner: String,
}
fn validate_has_prefix(value: &String) -> Result<(), ValidationError> {
    if value.starts_with("aad.") {
        Ok(())
    } else {
        Err(ValidationError::new("must start with 'aad.'"))
    }
}
impl AzureDevOpsEntraUserDescriptor {
    pub fn try_new(inner: impl Into<String>) -> eyre::Result<Self> {
        let inner = inner.into();
        let descriptor = Self { inner };
        descriptor.validate()?;
        Ok(descriptor)
    }
}

impl std::fmt::Display for AzureDevOpsEntraUserDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}
impl serde::Serialize for AzureDevOpsEntraUserDescriptor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AzureDevOpsEntraUserDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_new(value).map_err(|e| D::Error::custom(format!("{e:?}")))
    }
}
impl Deref for AzureDevOpsEntraUserDescriptor {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for AzureDevOpsEntraUserDescriptor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod test {
    use super::AzureDevOpsEntraUserDescriptor;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        AzureDevOpsEntraUserDescriptor::try_new("aad.bruh")?;
        Ok(())
    }
    #[test]
    pub fn it_works_2() -> eyre::Result<()> {
        assert!(AzureDevOpsEntraUserDescriptor::try_new("fail.bruh").is_err());
        Ok(())
    }

    #[test]
    pub fn serialization_works() -> eyre::Result<()> {
        let descriptor = AzureDevOpsEntraUserDescriptor::try_new("aad.bruh")?;
        let serialized = serde_json::to_string(&descriptor)?;
        assert_eq!(serialized, "\"aad.bruh\"");
        let deserialized: AzureDevOpsEntraUserDescriptor = serde_json::from_str(&serialized)?;
        assert_eq!(descriptor, deserialized);
        Ok(())
    }
}
