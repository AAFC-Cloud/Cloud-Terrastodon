use crate::QueryRestResultFormat;
use crate::ResourceGraphQueryRestScopeFilterOption;

/// [Resource Graph query options](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryrequestoptions).
#[derive(Debug, facet::Facet)]
pub struct ResourceGraphQueryRestOptions {
    #[facet(rename = "$skip")]
    pub skip: u64,
    #[facet(rename = "$top")]
    pub top: u64,
    #[facet(rename = "$skipToken")]
    pub skip_token: Option<String>,
    #[facet(rename = "authorizationScopeFilter")]
    pub authorization_scope_filter: ResourceGraphQueryRestScopeFilterOption,
    #[facet(rename = "resultFormat")]
    pub result_format: QueryRestResultFormat,
}
