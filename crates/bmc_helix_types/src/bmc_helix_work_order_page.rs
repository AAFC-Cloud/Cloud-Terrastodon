use crate::BmcHelixEntryLink;
use crate::BmcHelixWorkOrderEntry;
use facet::Facet;
use std::collections::BTreeMap;

/// Page returned by the [AR System form entries endpoint](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/AR-System/ars213/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/Syntax-and-schemes-for-the-REST-API/Pagination-schemes-for-REST-API-responses/).
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixWorkOrderPage {
    pub entries: Vec<BmcHelixWorkOrderEntry>,
    #[facet(rename = "_links")]
    pub links: Option<BTreeMap<String, Vec<BmcHelixEntryLink>>>,
    #[facet(rename = "numMatches")]
    pub num_matches: Option<usize>,
}
