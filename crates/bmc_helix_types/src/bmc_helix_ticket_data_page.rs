use crate::BmcHelixTicket;
use crate::BmcHelixTicketRow;
use facet::Facet;

/// [DataPage](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is251/Developing-applications-by-using-BMC-Helix-Innovation-Studio/Developing-and-deploying-code-based-applications/Customizing-an-application-with-REST-APIs/Retrieving-record-instances-using-REST-API/) envelope returned by RecordInstanceDataPageQuery.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixTicketDataPage {
    pub data: Vec<BmcHelixTicketRow>,
    /// The request disables the total count. Keep a reported value for wire compatibility.
    pub total_size: Option<i64>,
    /// Opaque cursor supplied by the Shared Ticket Console. The request
    /// paginates with the DataPageQuery's start index.
    pub cursor: Option<String>,
}

impl From<BmcHelixTicketDataPage> for Vec<BmcHelixTicket> {
    fn from(page: BmcHelixTicketDataPage) -> Self {
        page.data.into_iter().map(BmcHelixTicket::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BmcHelixTicketFieldId;
    use cloud_terrastodon_azure_types::ArbitraryJson;

    #[test]
    fn console_projection_keeps_numeric_fields_and_ticket_identity() -> eyre::Result<()> {
        #[derive(Facet)]
        struct ProjectedRow {
            #[facet(rename = "488100940")]
            custom_cell: Option<String>,
            #[facet(rename = "379")]
            record_id: String,
            #[facet(rename = "488100470")]
            ticket_number: String,
            #[facet(rename = "488100098")]
            status_code: i64,
            #[facet(rename = "488100080")]
            assignee: String,
        }

        #[derive(Facet)]
        #[facet(rename_all = "camelCase")]
        struct ConsolePage {
            total_size: Option<i64>,
            data: Vec<ProjectedRow>,
            cursor: String,
        }

        let response = ConsolePage {
            total_size: None,
            data: vec![ProjectedRow {
                custom_cell: None,
                record_id: "synthetic-record".to_owned(),
                ticket_number: "WO00001".to_owned(),
                status_code: 12,
                assignee: "synthetic-assignee".to_owned(),
            }],
            cursor: "synthetic-cursor".to_owned(),
        };
        let encoded = facet_json::to_vec(&response)?;
        let page: BmcHelixTicketDataPage = facet_json::from_slice(&encoded)?;
        let [ticket]: [BmcHelixTicket; 1] = Vec::from(page)
            .try_into()
            .map_err(|_| eyre::eyre!("expected one decoded ticket"))?;
        assert_eq!(ticket.id.as_str(), "synthetic-record");
        assert_eq!(ticket.display_id.as_deref(), Some("WO00001"));
        assert_eq!(ticket.assignee.as_deref(), Some("synthetic-assignee"));
        assert_eq!(
            ticket.fields.get(&BmcHelixTicketFieldId::STATUS),
            Some(&ArbitraryJson::try_from_facet(&12i64)?)
        );
        assert_eq!(
            ticket.fields.get(&BmcHelixTicketFieldId::new(488100940)),
            Some(&ArbitraryJson::try_from_facet(&None::<String>)?)
        );
        Ok(())
    }
}
