use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::HashMap;

/// A [Microsoft Graph error](https://learn.microsoft.com/graph/errors#error-resource-type) returned for a batch item.
#[derive(Debug, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct MicrosoftGraphBatchResponseEntryError {
    pub code: String,
    pub message: String,
    #[facet(rename = "innerError", alias = "innererror")]
    pub inner_error: Option<HashMap<String, ArbitraryJson>>,
    pub details: Option<Vec<Self>>,
}
