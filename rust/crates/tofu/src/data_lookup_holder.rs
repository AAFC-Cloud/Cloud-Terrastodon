use cloud_terrasotodon_core_azure::prelude::ScopeImpl;
use std::collections::HashMap;
use cloud_terrasotodon_core_tofu_types::prelude::TofuDataReference;

#[derive(Default)]
pub struct DataLookupHolder {
    pub data_references_by_id: HashMap<ScopeImpl, TofuDataReference>,
}
