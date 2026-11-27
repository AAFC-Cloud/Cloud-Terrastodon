use crate::BmcHelixTicketFieldId;
use crate::BmcHelixTicketRecordId;
use crate::bmc_helix_ticket_field_instance::BmcHelixTicketFieldInstance;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;
use std::collections::BTreeMap;

/// A projected [Innovation Suite record instance](https://docs.helixops.ai/bin/Service-Management/Enterprise-Service-Management/BMC-Helix-Business-Workflows/bwf233/Administering/Securing-access-and-addressing-privacy/Enabling-requester-identity-validation/).
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixTicketRecord {
    pub resource_type: String,
    pub id: BmcHelixTicketRecordId,
    pub display_id: Option<String>,
    pub record_definition_name: String,
    pub field_instances: BTreeMap<BmcHelixTicketFieldId, BmcHelixTicketFieldInstance>,
    /// The administrator defines which security labels and values can appear.
    pub permitted_groups_by_security_labels: Option<BTreeMap<String, ArbitraryJson>>,
    pub permitted_users_by_security_labels: Option<BTreeMap<String, ArbitraryJson>>,
    pub permitted_roles_by_security_labels: Option<BTreeMap<String, ArbitraryJson>>,
}
