use crate::config::Config;
use ordermap::OrderSet;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct WorkDirsConfig {
    pub work_dirs: OrderSet<PathBuf>,
}

#[async_trait::async_trait]
impl Config for WorkDirsConfig {
    const FILE_SLUG: &'static str = "work_dirs";
}
