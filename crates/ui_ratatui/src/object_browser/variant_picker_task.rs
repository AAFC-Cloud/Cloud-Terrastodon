use super::pickers::VariantPicker;
use crate::object_explorer::SlotId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(super) enum VariantPickerOutcome {
    Selected { slot: SlotId, variant: usize },
    Cancelled,
    Failed(String),
}

pub(super) struct VariantPickerTask {
    handle: JoinHandle<eyre::Result<VariantPickerOutcome>>,
    cancellation: CancellationToken,
}

impl VariantPickerTask {
    pub(super) fn spawn(
        picker: &VariantPicker,
        cancellation: &CancellationScope,
    ) -> Result<Self, String> {
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
        let initial_focused = choices
            .get(picker.selected_index())
            .map(|choice| choice.key.clone());
        let slot = picker.slot();
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let (cancellation, future) = cancellation.with_async_child_scope(move |scope| async move {
            let mut picker = PickerTui::new()
                .set_header("Pick Variant")
                .set_auto_accept(false)
                .terminal_coordinator(coordinator);
            if let Some(key) = initial_focused {
                picker = picker.set_initial_focused(key);
            }
            eyre::Ok(match picker.pick_one(choices, &scope).await {
                Ok(variant) => VariantPickerOutcome::Selected { slot, variant },
                Err(PickError::Cancelled) => VariantPickerOutcome::Cancelled,
                Err(error) => VariantPickerOutcome::Failed(error.to_string()),
            })
        });
        let handle = tokio::spawn(future);
        Ok(Self {
            handle,
            cancellation,
        })
    }

    pub(super) async fn close(self) {
        self.cancellation.request_cancel("object browser closed");
        match self.finish().await {
            Ok(VariantPickerOutcome::Failed(error)) | Err(error) => {
                tracing::warn!(%error, "Nested picker failed during shutdown");
            }
            Ok(_) => {}
        }
    }
    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<VariantPickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("variant PickerTui task failed: {error}"))?
            .map_err(|error| error.to_string())
    }
}
