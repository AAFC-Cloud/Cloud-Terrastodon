use crate::config::Config;
use facet::Facet;
use ordermap::OrderSet;
use std::path::PathBuf;

#[derive(Debug, Default, Facet, Clone, PartialEq)]
#[facet(opaque, proxy = WorkDirsConfigProxy)]
pub struct WorkDirsConfig {
    pub work_dirs: OrderSet<PathBuf>,
}

#[derive(Debug, Default, Facet, Clone, PartialEq)]
struct WorkDirsConfigProxy {
    pub work_dirs: Vec<PathBuf>,
}

impl From<WorkDirsConfigProxy> for WorkDirsConfig {
    fn from(value: WorkDirsConfigProxy) -> Self {
        Self {
            work_dirs: value.work_dirs.into_iter().collect(),
        }
    }
}

impl From<&WorkDirsConfig> for WorkDirsConfigProxy {
    fn from(value: &WorkDirsConfig) -> Self {
        Self {
            work_dirs: value.work_dirs.iter().cloned().collect(),
        }
    }
}

#[async_trait::async_trait]
impl Config for WorkDirsConfig {
    const FILE_SLUG: &'static str = "work_dirs";
}

cloud_terrastodon_registry::register_thing!(WorkDirsConfig);
