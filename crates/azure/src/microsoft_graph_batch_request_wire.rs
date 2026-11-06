use crate::microsoft_graph_batch_request_entry_wire::MicrosoftGraphBatchRequestEntryWire;
use facet::Facet;

#[derive(Debug, Facet)]
pub(crate) struct MicrosoftGraphBatchRequestWire<T> {
    pub(crate) requests: Vec<MicrosoftGraphBatchRequestEntryWire<T>>,
}
