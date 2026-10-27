use super::dump_plan_report::DumpPlanReport;
use super::dump_planner::DumpPlanner;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_excalidraw::ExcalidrawPlanningBoard;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Retains the terminal's accepted target selection and corresponding diagram.
/// A worker publishes a cloned session only after the shared-file transaction
/// succeeds, so a failed write cannot acknowledge a plan that is absent on disk.
#[derive(Clone)]
pub(in crate::cli::command) struct DumpPlanSession {
    planner: Arc<DumpPlanner>,
    report: DumpPlanReport,
    accepted_targets: Vec<String>,
}

#[cfg(test)]
#[path = "dump_plan_session_tests.rs"]
mod tests;

impl DumpPlanSession {
    pub(super) fn new(
        planner: DumpPlanner,
        report: DumpPlanReport,
        mut initial_targets: Vec<String>,
    ) -> Self {
        initial_targets.sort();
        initial_targets.dedup();
        Self {
            planner: Arc::new(planner),
            report,
            accepted_targets: initial_targets,
        }
    }

    pub(in crate::cli::command) fn refresh(
        &mut self,
        current: Option<&str>,
        _cancellation: &CancellationScope,
    ) -> Result<Option<String>> {
        let mut board = ExcalidrawPlanningBoard::initialize_diagram(current)?;
        board.set_plan(&self.report.plan)?;
        let updated = board.to_json()?;
        Ok((current != Some(updated.as_str())).then_some(updated))
    }

    pub(in crate::cli::command) fn targets(&self) -> &[String] {
        &self.accepted_targets
    }

    pub(in crate::cli::command) fn target_choices(&self) -> Vec<String> {
        self.planner
            .target_names()
            .iter()
            .chain(self.targets())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Replace the terminal selection and merge its diagram into the latest
    /// drawing. A caller publishes this cloned session only after the atomic
    /// write succeeds, retaining the previous plan while a file lock is busy.
    pub(in crate::cli::command) fn replace_targets(
        &mut self,
        current: &str,
        expected: &[String],
        replacement: &[String],
        cancellation: &CancellationScope,
    ) -> Result<Option<String>> {
        ensure!(
            self.accepted_targets == expected,
            "Plan targets changed while the picker was open; reopen the picker to edit the latest selection"
        );
        self.report.plan = self.planner.plan(replacement, cancellation)?;
        self.report.explain_assumptions();
        self.accepted_targets = replacement.to_vec();
        self.accepted_targets.sort();
        self.accepted_targets.dedup();
        self.refresh(Some(current), cancellation)
    }

    pub(in crate::cli::command) fn into_report(self) -> DumpPlanReport {
        self.report
    }
}
