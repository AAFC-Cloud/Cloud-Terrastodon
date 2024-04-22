use anyhow::Result;
use tokio::fs;

use crate::import_policy::import_policy;
use crate::process_generated::process_generated;
pub enum Action {
    ImportPolicy,
    ProcessGenerated,
}
impl Action {
    pub fn name(&self) -> &str {
        match self {
            Action::ImportPolicy => "import policy",
            Action::ProcessGenerated => "process generated",
        }
    }
    pub async fn invoke(&self) -> Result<()> {
        match self {
            Action::ImportPolicy => import_policy().await,
            Action::ProcessGenerated => process_generated().await,
        }
    }
    pub fn variants() -> [Action; 2] {
        [Action::ImportPolicy, Action::ProcessGenerated]
    }
    pub async fn is_available(&self) -> bool {
        match self {
            Action::ProcessGenerated => fs::try_exists("ignore/imports/generated.tf")
                .await
                .unwrap_or(false),
            _ => true,
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
