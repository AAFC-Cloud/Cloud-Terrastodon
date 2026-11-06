use crate::VirtualMachineSize;
use facet::Facet;
use std::sync::Arc;

/// A [Compute VM size collection](https://learn.microsoft.com/rest/api/compute/virtual-machine-sizes/list?view=rest-compute-2022-11-01#virtualmachinesizelistresult).
#[derive(Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct VirtualMachineSizeListResponse {
    pub value: Vec<Arc<VirtualMachineSize>>,
}
