use crate::CommandKind;
use bstr::BString;
use facet::Facet;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// The execution inputs that distinguish cached subprocess output.
#[derive(Facet)]
pub(crate) struct ProcessFingerprint {
    pub(crate) kind: CommandKind,
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) removed_env: BTreeSet<String>,
    pub(crate) removed_env_prefixes: BTreeSet<String>,
    pub(crate) run_dir: Option<String>,
    pub(crate) stdin_content: Option<String>,
    pub(crate) debug_inputs: BTreeMap<String, BString>,
}
