use cloud_terrastodon_azure_types::CostManagementQueryResult;
use cloud_terrastodon_rest::SerializableRestResponse;

/// A decoded successful query or the failed exchange needed for throttle recovery.
pub(crate) enum CostManagementResponse {
    Parsed(CostManagementQueryResult),
    Raw(SerializableRestResponse),
}
