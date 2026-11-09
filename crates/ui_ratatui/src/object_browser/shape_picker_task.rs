use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
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
    handle: JoinHandle<eyre::Result<ShapePickerOutcome>>,
    cancellation: CancellationToken,
}

impl ShapePickerTask {
    pub(super) fn spawn(cancellation: &CancellationScope) -> Result<Self, String> {
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
        let (cancellation, future) = cancellation.with_async_child_scope(move |scope| async move {
            eyre::Ok(
                match PickerTui::new()
                    .set_header("Pick Shape")
                    .set_auto_accept(false)
                    .terminal_coordinator(coordinator)
                    .pick_one(choices, &scope)
                    .await
                {
                    Ok(shape) => ShapePickerOutcome::Selected(shape),
                    Err(PickError::Cancelled) => ShapePickerOutcome::Cancelled,
                    Err(error) => ShapePickerOutcome::Failed(error.to_string()),
                },
            )
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
            Ok(ShapePickerOutcome::Failed(error)) | Err(error) => {
                tracing::warn!(%error, "Nested picker failed during shutdown");
            }
            Ok(_) => {}
        }
    }
    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<ShapePickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("shape PickerTui task failed: {error}"))?
            .map_err(|error| error.to_string())
    }
}
