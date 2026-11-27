use crate::BmcHelixEntryLink;
use crate::BmcHelixWorkOrder;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::BTreeMap;

/// One entry in an [AR System work-order query response](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is254/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixWorkOrderEntry {
    pub values: BmcHelixWorkOrder,
    #[facet(rename = "_links")]
    pub links: Option<BTreeMap<String, Vec<BmcHelixEntryLink>>>,
}
