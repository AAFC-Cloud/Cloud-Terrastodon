use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use crate::PathMapper;

#[derive(Debug, Clone)]
pub enum CommandArgument {
    /// Will be passed as-is to the command
    Literal(OsString),
    /// Will be replaced with the canonical path to the adjacent file with the same name
    DeferredAdjacentFilePath {
        key: PathBuf,
        mapper: Arc<dyn PathMapper>,
    },
}

impl From<CommandArgument> for OsString {
    fn from(arg: CommandArgument) -> Self {
        match arg {
            CommandArgument::Literal(lit) => lit,
            CommandArgument::DeferredAdjacentFilePath { key, mapper } => {
                mapper.map_path(key.as_path()).as_os_str().to_owned()
            }
        }
    }
}