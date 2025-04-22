use crate::iconfig::IConfig;
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
impl IConfig for WorkDirsConfig {
    const FILE_SLUG: &'static str = "work_dirs";
}

#[cfg(test)]
mod test {
    use ordermap::OrderSet;

    /// Indexmap does not check ordering for comparison, so we use ordermap instead.
    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let mut a = OrderSet::new();
        a.insert("a".to_string());
        a.insert("b".to_string());
        let mut b = a.clone();
        b.swap_indices(0,1);
        assert_ne!(a,b);
        Ok(())
    }
}