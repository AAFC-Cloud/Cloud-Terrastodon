use facet::Facet;

/// Filesystem metadata exposed to each picker query engine.
#[derive(Debug, Facet)]
pub(super) struct FsPickEntry {
    pub path: String,
    pub absolute_path: String,
    pub name: String,
    pub stem: Option<String>,
    pub extension: Option<String>,
    pub depth: usize,
    #[facet(rename = "type")]
    pub entry_type: &'static str,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
}
