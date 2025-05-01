use cloud_terrastodon_azure::prelude::ScopeImpl;
use cloud_terrastodon_hcl_types::prelude::HCLDataBlockReference;
use std::collections::HashMap;

#[derive(Default)]
pub struct DataLookupHolder {
    pub data_references_by_id: HashMap<ScopeImpl, HCLDataBlockReference>,
}
