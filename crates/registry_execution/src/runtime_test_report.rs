use crate::RuntimeTestCaseReport;
use crate::RuntimeTestResult;
use eyre::ensure;
use facet::Facet;

/// The complete disposition of a requested verification run.
#[derive(Debug, Facet)]
pub struct RuntimeTestReport {
    pub clean: bool,
    pub debug_decoding: bool,
    /// Cancellation may arrive during a final protected case which still finishes.
    pub cancelled: bool,
    pub cases: Vec<RuntimeTestCaseReport>,
}

impl RuntimeTestReport {
    /// The output boundary presents the report before propagating this status.
    pub fn outcome(&self) -> eyre::Result<()> {
        ensure!(!self.cancelled, "runtime verification was cancelled");
        ensure!(
            self.cases.iter().any(|case| matches!(
                case.result,
                RuntimeTestResult::Passed(_) | RuntimeTestResult::Failed { .. }
            )),
            "no runtime verification case completed"
        );
        ensure!(
            !self.cases.iter().any(|case| matches!(
                case.result,
                RuntimeTestResult::Failed { .. } | RuntimeTestResult::Cancelled { .. }
            )),
            "runtime verification failed or was cancelled"
        );
        Ok(())
    }
}
