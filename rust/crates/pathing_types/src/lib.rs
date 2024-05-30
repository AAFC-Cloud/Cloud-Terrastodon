use std::path::Path;
use std::path::PathBuf;

const IGNORE_ROOT: &str = "ignore";

pub enum IgnoreDir {
    Root,
    Commands,
    Imports,
    Processed,
}
impl From<IgnoreDir> for PathBuf {
    fn from(value: IgnoreDir) -> Self {
        match value {
            IgnoreDir::Root => PathBuf::from(IGNORE_ROOT),
            IgnoreDir::Commands => PathBuf::from_iter([IGNORE_ROOT, "commands"]),
            IgnoreDir::Imports => PathBuf::from_iter([IGNORE_ROOT, "imports"]),
            IgnoreDir::Processed => PathBuf::from_iter([IGNORE_ROOT, "processed"]),
        }
    }
}
impl IgnoreDir {
    pub fn join(self, path: impl AsRef<Path>) -> PathBuf {
        let buf: PathBuf = self.into();
        buf.join(path)
    }
}
