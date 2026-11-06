use crate::MicrosoftGraphBatchResponseEntry;
use cloud_terrastodon_command::FromCommandOutput;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

/// Responses to a [Microsoft Graph JSON batch](https://learn.microsoft.com/graph/json-batching#processing-the-json-batch-response).
#[derive(Debug, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct MicrosoftGraphBatchResponse<T> {
    pub responses: Vec<Arc<MicrosoftGraphBatchResponseEntry<T>>>,
}
impl<T: FromCommandOutput> MicrosoftGraphBatchResponse<T> {
    pub(crate) fn validate_request_ids(&self, expected: &[String]) -> eyre::Result<()> {
        eyre::ensure!(
            self.responses.len() == expected.len(),
            "Microsoft Graph batch returned a different number of responses"
        );
        for id in expected {
            eyre::ensure!(
                self.responses
                    .iter()
                    .filter(|response| response.id == *id)
                    .count()
                    == 1,
                "Microsoft Graph batch response IDs differ from its requests"
            );
        }
        Ok(())
    }
}
impl<T: FromCommandOutput> Deref for MicrosoftGraphBatchResponse<T> {
    type Target = Vec<Arc<MicrosoftGraphBatchResponseEntry<T>>>;
    fn deref(&self) -> &Self::Target {
        &self.responses
    }
}
impl<T: FromCommandOutput> DerefMut for MicrosoftGraphBatchResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.responses
    }
}
