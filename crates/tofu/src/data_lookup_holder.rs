use cloud_terrastodon_azure::prelude::ScopeImpl;
use cloud_terrastodon_tofu_types::prelude::TofuDataReference;
use std::collections::HashMap;

#[derive(Default)]
pub struct DataLookupHolder {
    pub data_references_by_id: HashMap<ScopeImpl, TofuDataReference>,
}
