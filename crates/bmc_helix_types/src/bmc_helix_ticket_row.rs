use crate::BmcHelixTicket;
use crate::bmc_helix_ticket_projection::BmcHelixTicketProjection;
use crate::bmc_helix_ticket_record::BmcHelixTicketRecord;
use facet::Facet;

/// A row in a ticket DataPage. The Shared Ticket Console exposes flat
/// projections alongside the standard Innovation Suite record representation.
#[derive(Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum BmcHelixTicketRow {
    Projection(BmcHelixTicketProjection),
    Record(BmcHelixTicketRecord),
}

impl From<BmcHelixTicketRow> for BmcHelixTicket {
    fn from(row: BmcHelixTicketRow) -> Self {
        match row {
            BmcHelixTicketRow::Projection(projection) => projection.into(),
            BmcHelixTicketRow::Record(record) => record.into(),
        }
    }
}
