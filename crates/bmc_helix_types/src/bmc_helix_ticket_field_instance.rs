use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;

/// A [record field instance](https://docs.helixops.ai/bin/Service-Management/Enterprise-Service-Management/BMC-Helix-Business-Workflows/bwf233/Administering/Securing-access-and-addressing-privacy/Enabling-requester-identity-validation/) returned for a projected ticket column.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixTicketFieldInstance {
    pub resource_type: String,
    pub id: u32,
    /// CRD columns own their value schema, and administrator-defined columns
    /// need not use the same value type as any built-in column.
    pub value: ArbitraryJson,
}
