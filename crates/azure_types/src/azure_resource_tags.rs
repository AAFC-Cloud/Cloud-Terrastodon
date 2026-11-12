use crate::AzureResourceTagsProperties;
use crate::ResourceTagsId;
use crate::ResourceType;
use crate::Scope;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::ensure;
use facet::Facet;

/// ARM tags resource returned by GET and PATCH at a scope.
/// The documented response examples omit id, name, and type.
/// [TagsResource schema](https://learn.microsoft.com/rest/api/resources/tags/get-at-scope?view=rest-resources-2021-04-01#tagsresource).
#[derive(Debug, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceTags {
    pub id: Option<ResourceTagsId>,
    pub name: Option<String>,
    #[facet(rename = "type")]
    pub kind: Option<ResourceType>,
    pub properties: AzureResourceTagsProperties,
}
impl<'a> Arbitrary<'a> for AzureResourceTags {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            id: Arbitrary::arbitrary(u)?,
            name: bool::arbitrary(u)?.then(|| "default".to_owned()),
            kind: bool::arbitrary(u)?.then_some(ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_TAGS),
            properties: Arbitrary::arbitrary(u)?,
        })
    }
}
impl AzureResourceTags {
    /// Check identity metadata when the service includes it.
    pub fn validate_identity(&self, expected: &ResourceTagsId) -> eyre::Result<()> {
        ensure!(
            self.id.as_ref().is_none_or(|id| id
                .expanded_form()
                .eq_ignore_ascii_case(&expected.expanded_form())),
            "tag response belongs to another resource"
        );
        ensure!(
            self.name.as_deref().is_none_or(|name| name == "default"),
            "tag response has an unexpected wrapper name"
        );
        ensure!(
            self.kind
                .as_ref()
                .is_none_or(|kind| *kind == ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_TAGS),
            "tag response has an unexpected resource type"
        );
        Ok(())
    }
}
cloud_terrastodon_registry::register_thing!(AzureResourceTags);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceTags);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn identity_validation_accepts_missing_metadata_but_rejects_other_resources() -> eyre::Result<()>
    {
        let expected: ResourceTagsId = "/subscriptions/00000000-0000-4000-8000-000000000001/providers/Microsoft.Resources/tags/default".parse()?;
        let mut tags = AzureResourceTags {
            id: None,
            name: None,
            kind: None,
            properties: AzureResourceTagsProperties {
                tags: HashMap::new(),
            },
        };
        tags.validate_identity(&expected)?;
        tags.id = Some(expected.expanded_form().to_ascii_uppercase().parse()?);
        tags.name = Some("default".to_owned());
        tags.kind = Some(ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_TAGS);
        tags.validate_identity(&expected)?;
        tags.id = Some("/subscriptions/00000000-0000-4000-8000-000000000002/providers/Microsoft.Resources/tags/default".parse()?);
        assert!(tags.validate_identity(&expected).is_err());
        Ok(())
    }
}
