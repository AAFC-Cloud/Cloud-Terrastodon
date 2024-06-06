use std::path::PathBuf;

pub struct CodeReference {
    pub display: String,
    pub line_number: usize,
    pub path: PathBuf,
}
impl std::fmt::Display for CodeReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} | {}", self.path.display(), self.display))
    }
}