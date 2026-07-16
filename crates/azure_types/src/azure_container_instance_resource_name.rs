use crate::slug::Slug;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use compact_str::CompactString;
use eyre::Context;
use eyre::bail;
use std::ops::Deref;
use std::str::FromStr;

/// Azure Container Instance container group names are 1-63 characters and use
/// lowercase letters, numbers, and hyphens.
///
/// <https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftcontainerinstance>
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet)]
#[facet(json::proxy = String)]
pub struct AzureContainerInstanceResourceName {
    inner: CompactString,
}
crate::impl_facet_string_proxy!(AzureContainerInstanceResourceName, value => value.to_string());

impl Slug for AzureContainerInstanceResourceName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_container_instance_resource_name(&inner)?;
        Ok(Self { inner })
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_container_instance_resource_name(&self.inner)
    }
}

fn validate_container_instance_resource_name(value: &CompactString) -> eyre::Result<()> {
    let char_count = value.chars().count();
    if !(1..=63).contains(&char_count) {
        bail!("Container instance resource name must be between 1 and 63 characters");
    }

    for (index, character) in value.chars().enumerate() {
        if !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-') {
            bail!(
                "Char {} at position {} in {:?} must be a lowercase letter, number, or hyphen",
                character,
                index,
                value
            );
        }
        if (index == 0 || index == char_count - 1) && !character.is_ascii_alphanumeric() {
            bail!(
                "Char {} at position {} in {:?} must be alphanumeric at the beginning and end",
                character,
                index,
                value
            );
        }
    }

    Ok(())
}

impl FromStr for AzureContainerInstanceResourceName {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for AzureContainerInstanceResourceName {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl std::fmt::Display for AzureContainerInstanceResourceName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.inner)
    }
}

impl Deref for AzureContainerInstanceResourceName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFrom<CompactString> for AzureContainerInstanceResourceName {
    type Error = eyre::Error;

    fn try_from(value: CompactString) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<AzureContainerInstanceResourceName> for CompactString {
    fn from(value: AzureContainerInstanceResourceName) -> Self {
        value.inner
    }
}

impl<'a> Arbitrary<'a> for AzureContainerInstanceResourceName {
    fn arbitrary(unstructured: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let length = unstructured.int_in_range(1..=63)?;
        let choices = ('a'..='z').chain('0'..='9').collect::<Vec<_>>();
        let middle_choices = choices.iter().copied().chain(['-']).collect::<Vec<_>>();

        let mut value = String::with_capacity(length);
        value.push(*unstructured.choose(&choices)?);
        if length > 1 {
            for _ in 0..(length - 2) {
                value.push(*unstructured.choose(&middle_choices)?);
            }
            value.push(*unstructured.choose(&choices)?);
        }

        Self::try_new(CompactString::from(value))
            .wrap_err("Failed to generate an Azure container instance resource name")
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

cloud_terrastodon_registry::register_thing!(AzureContainerInstanceResourceName);
cloud_terrastodon_registry::register_arbitrary!(AzureContainerInstanceResourceName);
