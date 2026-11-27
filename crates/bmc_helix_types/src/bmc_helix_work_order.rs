use crate::BmcHelixWorkOrderId;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;
use std::collections::BTreeMap;

/// Values returned by the [work-order interface form](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Service-Request-Management/srm2002/Integrating/Web-services-for-Work-Order-Management/GetListOfWorkOrders/).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixWorkOrder {
    #[facet(rename = "Work Order ID")]
    pub id: BmcHelixWorkOrderId,
    /// Other AR form fields, which can be customized on the server.
    #[facet(flatten)]
    pub values: BTreeMap<String, ArbitraryJson>,
}

cloud_terrastodon_registry::register_thing!(BmcHelixWorkOrder);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixWorkOrder);
