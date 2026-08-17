use cloud_terrastodon_user_input::{Choice, PickError, PickerTui, TerminalCoordinator};
use tokio::task::JoinHandle;

use super::pickers::VariantPicker;
use crate::object_explorer::SlotId;

#[derive(Debug)]
pub(super) enum VariantPickerOutcome {
    Selected { slot: SlotId, variant: usize },
    Cancelled,
    Failed(String),
}

pub(super) struct VariantPickerTask {
    handle: JoinHandle<VariantPickerOutcome>,
}

impl VariantPickerTask {
    pub(super) fn spawn(picker: &VariantPicker) -> Result<Self, String> {
        let choices = picker
            .matches()
            .map(|(variant, label)| Choice {
                key: label.to_owned(),
                value: variant,
            })
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err("the enum has no variants".to_owned());
        }
        let slot = picker.slot();
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header("Pick Variant")
                .set_auto_accept(false)
                .terminal_coordinator(coordinator)
                .pick_one(choices)
                .await
            {
                Ok(variant) => VariantPickerOutcome::Selected { slot, variant },
                Err(PickError::Cancelled) => VariantPickerOutcome::Cancelled,
                Err(error) => VariantPickerOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self { handle })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<VariantPickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("variant PickerTui task failed: {error}"))
    }
}
