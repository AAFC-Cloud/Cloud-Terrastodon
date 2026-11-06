use crate::microsoft_graph_batch_request_entry_wire::MicrosoftGraphBatchRequestEntryWire;

#[derive(Debug, facet::Facet)]
pub(crate) struct MicrosoftGraphBatchRequestWire<T> {
    pub(crate) requests: Vec<MicrosoftGraphBatchRequestEntryWire<T>>,
}
