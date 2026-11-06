use crate::MicrosoftGraphBatchResponseEntryBody;
use cloud_terrastodon_command::FromCommandOutput;
use eyre::bail;
use facet::Facet;
use std::collections::HashMap;

/// One response in a [Microsoft Graph JSON batch](https://learn.microsoft.com/graph/json-batching#processing-the-json-batch-response).
#[derive(Debug, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct MicrosoftGraphBatchResponseEntry<T> {
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: MicrosoftGraphBatchResponseEntryBody<T>,
}
impl<T: FromCommandOutput> MicrosoftGraphBatchResponseEntry<T> {
    /// Borrow a successful body without copying the cached entry or its payload.
    pub fn body(&self) -> eyre::Result<&T> {
        match &self.body {
            MicrosoftGraphBatchResponseEntryBody::Success(t) => Ok(t),
            MicrosoftGraphBatchResponseEntryBody::Error { error } => {
                bail!(
                    "Microsoft Graph API error for request {} (status {}): {} - {}",
                    self.id,
                    self.status,
                    error.code,
                    error.message
                );
            }
        }
    }
}
