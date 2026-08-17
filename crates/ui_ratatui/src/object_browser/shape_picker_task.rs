use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(super) enum ShapePickerOutcome {
    Selected(String),
    Cancelled,
    Failed(String),
}

pub(super) struct ShapePickerTask {
    handle: JoinHandle<ShapePickerOutcome>,
}

impl ShapePickerTask {
    pub(super) fn spawn() -> Result<Self, String> {
        let choices = cloud_terrastodon_registry::known_shapes()
            .into_iter()
            .map(|shape| Choice {
                key: shape.label.clone(),
                value: shape.label,
            })
            .collect::<Vec<_>>();
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header("Pick Shape")
                .set_auto_accept(false)
                .terminal_coordinator(coordinator)
                .pick_one(choices)
                .await
            {
                Ok(shape) => ShapePickerOutcome::Selected(shape),
                Err(PickError::Cancelled) => ShapePickerOutcome::Cancelled,
                Err(error) => ShapePickerOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self { handle })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<ShapePickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("shape PickerTui task failed: {error}"))
    }
}
