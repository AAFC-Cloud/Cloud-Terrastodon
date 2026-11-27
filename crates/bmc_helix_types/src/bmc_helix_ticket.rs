use crate::BmcHelixTicketFieldId;
use crate::BmcHelixTicketProjection;
use crate::BmcHelixTicketRecordId;
use crate::bmc_helix_ticket_record::BmcHelixTicketRecord;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;
use jiff::Timestamp;
use std::collections::BTreeMap;

/// A result from the [Shared Ticket Console record definition](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Helix-ITSM/itsm262/Administering/Enabling-and-customizing-Shared-Ticket-Console/Customizing-Shared-Ticket-Console/).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
pub struct BmcHelixTicket {
    pub id: BmcHelixTicketRecordId,
    pub display_id: Option<String>,
    pub title: Option<String>,
    /// Detailed description from the console's record definition, when available.
    pub description: Option<String>,
    pub created_at: Option<Timestamp>,
    /// The last modification reported by the console.
    pub last_activity_at: Option<Timestamp>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    /// Values retain their numeric field IDs even when a display name is known.
    pub fields: BTreeMap<BmcHelixTicketFieldId, ArbitraryJson>,
}

impl From<BmcHelixTicketRecord> for BmcHelixTicket {
    fn from(record: BmcHelixTicketRecord) -> Self {
        let fields = record
            .field_instances
            .into_iter()
            .map(|(id, field)| (id, field.value))
            .collect();
        Self::from_fields(record.id, record.display_id, fields)
    }
}

impl From<BmcHelixTicketProjection> for BmcHelixTicket {
    fn from(projection: BmcHelixTicketProjection) -> Self {
        let mut fields = projection.fields;
        fields.insert(
            BmcHelixTicketFieldId::RECORD_ID,
            ArbitraryJson::from_text(projection.id.as_str()),
        );
        let display_id = fields
            .get(&BmcHelixTicketFieldId::TICKET_NUMBER)
            .and_then(ticket_field_text);
        Self::from_fields(projection.id, display_id, fields)
    }
}

impl BmcHelixTicket {
    fn from_fields(
        id: BmcHelixTicketRecordId,
        display_id: Option<String>,
        fields: BTreeMap<BmcHelixTicketFieldId, ArbitraryJson>,
    ) -> Self {
        Self {
            id,
            display_id,
            title: fields
                .get(&BmcHelixTicketFieldId::SUMMARY)
                .and_then(ticket_field_text),
            description: None,
            created_at: None,
            last_activity_at: None,
            status: fields
                .get(&BmcHelixTicketFieldId::STATUS)
                .and_then(ticket_field_text),
            assignee: fields
                .get(&BmcHelixTicketFieldId::ASSIGNEE)
                .and_then(ticket_field_text),
            fields,
        }
    }

    pub fn field_text(&self, id: BmcHelixTicketFieldId) -> Option<String> {
        self.fields.get(&id).and_then(ticket_field_text)
    }

    pub fn field_timestamp(&self, id: BmcHelixTicketFieldId) -> Option<Timestamp> {
        self.field_text(id)?.parse().ok()
    }
}

fn ticket_field_text(value: &ArbitraryJson) -> Option<String> {
    let value = value.as_ref();
    // Facet accepts numeric JSON as text. Project only actual JSON strings into
    // named text fields and preserve other values in the numeric projection.
    if value.trim_start().starts_with('"') {
        facet_json::from_str::<String>(value).ok()
    } else {
        None
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixTicket);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixTicket);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BmcHelixTicketFieldInstance;

    #[test]
    fn labeled_columns_preserve_unrecognized_value_types() -> eyre::Result<()> {
        let status_code = ArbitraryJson::try_from_facet(&12u32)?;
        let record = BmcHelixTicketRecord {
            resource_type: "test record instance".to_owned(),
            id: "test-ticket".to_owned().try_into()?,
            display_id: None,
            record_definition_name: "test record definition".to_owned(),
            field_instances: BTreeMap::from([
                (
                    BmcHelixTicketFieldId::ASSIGNEE,
                    BmcHelixTicketFieldInstance {
                        resource_type: "test field instance".to_owned(),
                        id: 488100080,
                        value: ArbitraryJson::from_text("test-user"),
                    },
                ),
                (
                    BmcHelixTicketFieldId::STATUS,
                    BmcHelixTicketFieldInstance {
                        resource_type: "test field instance".to_owned(),
                        id: 488100098,
                        value: status_code.clone(),
                    },
                ),
            ]),
            permitted_groups_by_security_labels: None,
            permitted_users_by_security_labels: None,
            permitted_roles_by_security_labels: None,
        };

        let ticket = BmcHelixTicket::from(record);
        assert_eq!(ticket.assignee.as_deref(), Some("test-user"));
        assert_eq!(ticket.status, None);
        assert_eq!(
            ticket.fields.get(&BmcHelixTicketFieldId::STATUS),
            Some(&status_code)
        );
        Ok(())
    }
}
