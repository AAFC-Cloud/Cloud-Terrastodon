use crate::AzureDevOpsWorkItemFields;
use crate::AzureDevOpsWorkItemId;
use crate::AzureDevOpsWorkItemRelation;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use eyre::Result;
use std::collections::BTreeMap;
use url::Url;

/// Full item response. None for relations means the response was not expanded.
/// https://learn.microsoft.com/rest/api/azure/devops/wit/work-items/get-work-item
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItem {
    pub id: AzureDevOpsWorkItemId,
    pub rev: i32,
    pub url: Url,
    #[facet(default)]
    pub fields: AzureDevOpsWorkItemFields,
    pub relations: Option<Vec<AzureDevOpsWorkItemRelation>>,
    #[facet(rename = "_links")]
    pub links: Option<ArbitraryJson>,
    pub comment_version_ref: Option<ArbitraryJson>,
    #[facet(default)]
    pub multiline_fields_format: BTreeMap<String, String>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsWorkItem {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            id: AzureDevOpsWorkItemId::arbitrary(u)?,
            rev: i32::arbitrary(u)?,
            url: Url::parse("https://dev.azure.com/example/_apis/wit/workitems/1")
                .expect("static URL is valid"),
            fields: AzureDevOpsWorkItemFields::arbitrary(u)?,
            relations: Option::<Vec<AzureDevOpsWorkItemRelation>>::arbitrary(u)?,
            links: Option::<ArbitraryJson>::arbitrary(u)?,
            comment_version_ref: Option::<ArbitraryJson>::arbitrary(u)?,
            multiline_fields_format: BTreeMap::<String, String>::arbitrary(u)?,
        })
    }
}

impl AzureDevOpsWorkItem {
    pub fn string_field(&self, field: &str) -> Result<String> {
        let value = self
            .fields
            .field(field)?
            .ok_or_else(|| eyre::eyre!("Missing field {field}"))?;
        facet_json::from_str(value.as_ref())
            .map_err(|_| eyre::eyre!("Expected a string in {field}"))
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItem);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItem);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsWorkItem>);
