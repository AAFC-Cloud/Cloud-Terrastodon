use crate::config::Config;
use ordermap::OrderSet;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkDirsConfig {
    pub work_dirs: OrderSet<PathBuf>,
}

impl Default for WorkDirsConfig {
    fn default() -> Self {
        Self {
            work_dirs: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl Config for WorkDirsConfig {
    const FILE_SLUG: &'static str = "work_dirs";
}
