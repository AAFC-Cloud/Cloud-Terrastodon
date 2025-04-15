use crate::iconfig::IConfig;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkDirsConfig {
    pub work_dirs: HashSet<PathBuf>,
}

impl Default for WorkDirsConfig {
    fn default() -> Self {
        Self {
            work_dirs: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl IConfig for WorkDirsConfig {
    const FILE_SLUG: &'static str = "work_dirs";
}
