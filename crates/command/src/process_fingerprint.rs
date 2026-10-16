use bstr::BString;
use std::collections::BTreeMap;

/// The execution inputs that distinguish cached subprocess output.
#[derive(facet::Facet)]
pub(crate) struct ProcessFingerprint {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) run_dir: Option<String>,
    pub(crate) stdin_content: Option<String>,
    pub(crate) debug_inputs: BTreeMap<String, BString>,
}
