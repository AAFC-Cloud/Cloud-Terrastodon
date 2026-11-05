use crate::MicrosoftGraphBatchResponseEntryError;

#[derive(Debug, facet::Facet)]
pub(crate) struct MicrosoftGraphBatchErrorEnvelope {
    pub error: MicrosoftGraphBatchResponseEntryError,
}
