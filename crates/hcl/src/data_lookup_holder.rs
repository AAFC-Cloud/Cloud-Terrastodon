use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_hcl_types::DataBlockReference;
use std::collections::HashMap;

#[derive(Default)]
pub struct DataLookupHolder {
    pub data_references_by_id: HashMap<ScopeImpl, DataBlockReference>,
}
