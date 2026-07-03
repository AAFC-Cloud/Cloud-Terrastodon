use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsEntraUserDescriptor(String);
fn validate_has_prefix(value: &str) -> eyre::Result<()> {
    if value.starts_with("aad.") {
        Ok(())
    } else {
        eyre::bail!(
            "Entra user descriptor must start with 'aad.', got '{}'",
            value
        );
    }
}
impl AzureDevOpsEntraUserDescriptor {
    pub fn try_new(inner: impl Into<String>) -> eyre::Result<Self> {
        let inner = inner.into();
        validate_has_prefix(&inner)?;
        Ok(Self(inner))
    }
}

impl std::fmt::Display for AzureDevOpsEntraUserDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Deref for AzureDevOpsEntraUserDescriptor {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsEntraUserDescriptor);

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
        let serialized = facet_json::to_string(&descriptor)?;
        assert_eq!(serialized, "\"aad.bruh\"");
        let deserialized: AzureDevOpsEntraUserDescriptor = facet_json::from_str(&serialized)?;
        assert_eq!(descriptor, deserialized);
        Ok(())
    }
}
