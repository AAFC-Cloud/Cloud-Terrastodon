use crate::BmcHelixTicketFieldId;
use crate::BmcHelixTicketRecordId;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;
use std::collections::BTreeMap;

/// A flat row returned by the [Shared Ticket Console's custom record definition](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Helix-ITSM/itsm262/Administering/Enabling-and-customizing-Shared-Ticket-Console/Customizing-Shared-Ticket-Console/).
/// The console projects cells directly onto the row under their numeric IDs.
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixTicketProjection {
    /// The console returns the unique record identifier in field 379.
    #[facet(rename = "379")]
    pub id: BmcHelixTicketRecordId,
    /// Projected fields may be absent or null and may have custom value types.
    #[facet(flatten, default)]
    pub fields: BTreeMap<BmcHelixTicketFieldId, ArbitraryJson>,
}
