use azure_types::scopes::ScopeImpl;
use std::collections::HashMap;
use tofu_types::prelude::TofuDataReference;

#[derive(Default)]
pub struct DataLookupHolder {
    pub data_references_by_id: HashMap<ScopeImpl, TofuDataReference>,
}
