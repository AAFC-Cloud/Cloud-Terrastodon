use crate::bmc_helix_ticket_query_values::BmcHelixTicketQueryValues;
use facet::Facet;

/// [Innovation Suite DataPageQuery POST](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is251/Getting-started/Tutorial/Module-2-Extending-your-Solution-in-Code/Building-custom-Service-Extensions-in-Java-for-the-Meal-Program-Library/A-custom-DataPageQuery-for-the-MealOrderService/) envelope.
#[derive(Facet)]
pub(crate) struct BmcHelixTicketQueryBody<'a> {
    pub values: BmcHelixTicketQueryValues<'a>,
}
