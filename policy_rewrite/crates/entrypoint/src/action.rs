use anyhow::Result;
use tokio::fs;
use tracing::instrument;
use crate::build_policy_imports::build_policy_imports;
use crate::process_generated::process_generated;
use crate::run_tf_import::run_tf_import;
#[derive(Debug)]
pub enum Action {
    BuildPolicyImports,
    RunTFImport,
    ProcessGenerated,
}
impl Action {
    pub fn name(&self) -> &str {
        match self {
            Action::BuildPolicyImports => "build policy imports",
            Action::RunTFImport => "perform import",
            Action::ProcessGenerated => "process generated",
        }
    }
    #[instrument]
    pub async fn invoke(&self) -> Result<()> {
        match self {
            Action::BuildPolicyImports => build_policy_imports().await,
            Action::RunTFImport => run_tf_import().await,
            Action::ProcessGenerated => process_generated().await,
        }
    }
    pub fn variants() -> Vec<Action> {
        vec![
            Action::BuildPolicyImports,
            Action::RunTFImport,
            Action::ProcessGenerated,
        ]
    }

    /// Some actions don't make sense if files are missing from expected locations.
    pub async fn is_available(&self) -> bool {
        match self {
            Action::RunTFImport => fs::try_exists("ignore/imports/imports.tf")
                .await
                .unwrap_or(false),
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
