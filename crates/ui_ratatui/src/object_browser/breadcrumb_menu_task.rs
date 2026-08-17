use cloud_terrastodon_user_input::{Choice, PickError, PickerTui, TerminalCoordinator};
use tokio::task::JoinHandle;

use super::pickers::{BreadcrumbPicker, BreadcrumbPickerChoice};

#[derive(Debug)]
pub(super) enum BreadcrumbMenuOutcome {
    Selected(BreadcrumbPickerChoice),
    Cancelled,
    Failed(String),
}

/// Nested fuzzy picker for choosing the next breadcrumb operation.
///
/// The picker owns only cloned menu metadata. The arena and query remain with
/// the explorer engine while TerminalCoordinator temporarily hands the
/// terminal to PickerTui.
pub(super) struct BreadcrumbMenuTask {
    handle: JoinHandle<BreadcrumbMenuOutcome>,
}

impl BreadcrumbMenuTask {
    pub(super) fn spawn(picker: &BreadcrumbPicker) -> Result<Self, String> {
        let choices = picker
            .rows()
            .iter()
            .map(|row| Choice {
                key: row.label().to_owned(),
                value: row.choice().clone(),
            })
            .collect::<Vec<_>>();
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header("Add Breadcrumb")
                .set_auto_accept(false)
                .terminal_coordinator(coordinator)
                .pick_one(choices)
                .await
            {
                Ok(selected) => BreadcrumbMenuOutcome::Selected(selected),
                Err(PickError::Cancelled) => BreadcrumbMenuOutcome::Cancelled,
                Err(error) => BreadcrumbMenuOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self { handle })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<BreadcrumbMenuOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("breadcrumb menu PickerTui task failed: {error}"))
    }
}
