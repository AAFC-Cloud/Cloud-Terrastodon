use cloud_terrastodon_bmc_helix_types::BmcHelixTicket;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketFieldId;
use eyre::ContextCompat;
use eyre::Report;
use eyre::Result;
use facet_value::Value;

/// Field IDs for the console's [title, description, and dates](https://docs.helixops.ai/bin/view/Service-Management/IT-Service-Management/BMC-Helix-ITSM/itsm261/Navigating-common-interfaces/Managing-your-work-by-using-Ticket-Console/).
#[derive(Default)]
pub(crate) struct BmcHelixTicketColumnFields {
    title: Option<BmcHelixTicketFieldId>,
    description: Option<BmcHelixTicketFieldId>,
    created_at: Option<BmcHelixTicketFieldId>,
    last_activity_at: Option<BmcHelixTicketFieldId>,
}

impl BmcHelixTicketColumnFields {
    pub fn ids(&self) -> impl Iterator<Item = BmcHelixTicketFieldId> {
        [
            self.title,
            self.created_at,
            self.last_activity_at,
            self.description,
        ]
        .into_iter()
        .flatten()
    }

    pub fn project(&self, mut ticket: BmcHelixTicket) -> BmcHelixTicket {
        if let Some(field) = self.title {
            ticket.title = ticket.field_text(field);
        }
        ticket.description = self
            .description
            .and_then(|field| ticket.field_text(field))
            .filter(|description| !description.trim().is_empty());
        ticket.created_at = self
            .created_at
            .and_then(|field| ticket.field_timestamp(field));
        ticket.last_activity_at = self
            .last_activity_at
            .and_then(|field| ticket.field_timestamp(field));
        ticket
    }
}

impl TryFrom<Value> for BmcHelixTicketColumnFields {
    type Error = Report;

    fn try_from(definition: Value) -> Result<Self> {
        // Each field subtype can carry its own configuration in the record
        // definition. Only the documented field identity participates here.
        let fields = definition
            .as_object()
            .and_then(|definition| definition.get("fieldDefinitions"))
            .and_then(Value::as_array)
            .wrap_err("Helix record definition did not include a fieldDefinitions array")?;
        let mut columns = Self::default();
        let mut short_description = None;
        for field in fields.iter() {
            let field = field
                .as_object()
                .wrap_err("Helix record definition contained a non-object field")?;
            let name = field
                .get("name")
                .and_then(Value::as_string)
                .wrap_err("Helix record definition contained a field without a name")?
                .as_str();
            let slot = if name.eq_ignore_ascii_case("Summary") {
                &mut columns.title
            } else if name.eq_ignore_ascii_case("Description") {
                &mut short_description
            } else if name.eq_ignore_ascii_case("Detailed_description") {
                &mut columns.description
            } else if name.eq_ignore_ascii_case("Create_date") {
                &mut columns.created_at
            } else if name.eq_ignore_ascii_case("Last_modified_date") {
                &mut columns.last_activity_at
            } else {
                continue;
            };
            let id = field
                .get("id")
                .and_then(Value::as_number)
                .and_then(|number| number.to_u32())
                .wrap_err_with(|| format!("Helix field {name} did not specify a numeric ID"))?;
            eyre::ensure!(
                slot.is_none(),
                "Helix record definition repeated the {name} field"
            );
            *slot = Some(BmcHelixTicketFieldId::new(id));
        }
        // The standard console record exposes the title under Description.
        columns.title = columns.title.or(short_description);
        Ok(columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_bmc_helix_types::BmcHelixTicketProjection;

    #[test]
    fn ticket_details_follow_the_record_definition() -> Result<()> {
        let definition = facet_value::value!({
            "fieldDefinitions": [
                {"name": "Last_modified_date", "id": 701},
                {"name": "Targetdate", "id": 702},
                {"name": "Create_date", "id": 703},
                {"name": "Description", "id": 704},
                {"name": "Detailed_description", "id": 705}
            ]
        });
        let columns = BmcHelixTicketColumnFields::try_from(definition)?;
        let raw_timestamp = "2026-01-01T09:00:00Z";
        let raw_last_modified = "2026-01-02T12:00:00Z";
        let description = "Synthetic description\nSecond line";
        let ticket = BmcHelixTicket::from(BmcHelixTicketProjection {
            id: "synthetic-record".to_owned().try_into()?,
            fields: [
                (701, raw_last_modified),
                (702, "2026-01-03T12:00:00Z"),
                (703, raw_timestamp),
                (704, "Synthetic title"),
                (705, description),
            ]
            .into_iter()
            .map(|(id, value)| {
                eyre::Ok((
                    BmcHelixTicketFieldId::new(id),
                    facet_json::from_str(&facet_json::to_string(value)?)?,
                ))
            })
            .collect::<Result<_>>()?,
        });
        let ticket = columns.project(ticket);
        assert_eq!(ticket.title.as_deref(), Some("Synthetic title"));
        assert_eq!(ticket.description.as_deref(), Some(description));
        assert_eq!(ticket.created_at, Some(raw_timestamp.parse()?));
        assert_eq!(ticket.last_activity_at, Some(raw_last_modified.parse()?));
        assert_eq!(
            columns.ids().collect::<Vec<_>>(),
            [704, 703, 701, 705].map(BmcHelixTicketFieldId::new),
        );
        Ok(())
    }
}
