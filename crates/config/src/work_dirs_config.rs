use crate::config::Config;
use facet::Facet;
use ordermap::OrderSet;
use std::convert::Infallible;
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

impl TryFrom<WorkDirsConfigProxy> for WorkDirsConfig {
    type Error = Infallible;

    fn try_from(value: WorkDirsConfigProxy) -> Result<Self, Self::Error> {
        Ok(Self {
            work_dirs: value.work_dirs.into_iter().collect(),
        })
    }
}

impl TryFrom<&WorkDirsConfig> for WorkDirsConfigProxy {
    type Error = Infallible;

    fn try_from(value: &WorkDirsConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            work_dirs: value.work_dirs.iter().cloned().collect(),
        })
    }
}

#[async_trait::async_trait]
impl Config for WorkDirsConfig {
    const FILE_SLUG: &'static str = "work_dirs";
}
