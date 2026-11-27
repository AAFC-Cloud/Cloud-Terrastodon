use facet::Facet;

/// Parameters for the Shared Ticket Console's RecordInstanceDataPageQuery.
///
/// The browser sends the values as strings, including numeric paging parameters.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub(crate) struct BmcHelixTicketQueryValues<'a> {
    pub data_page_type: &'static str,
    pub page_size: String,
    pub start_index: String,
    #[facet(rename = "recorddefinition")]
    pub record_definition: &'static str,
    pub should_include_total_size: &'static str,
    pub property_selection: &'a str,
    pub sort_by: String,
    pub query_expression: &'a str,
}
