use crate::object_explorer::SlotId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_registry::RegisteredShape;
use cloud_terrastodon_registry::known_functions;
use cloud_terrastodon_registry::known_things;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use tokio::task::JoinHandle;

/// A terminal selection of registry identities, producing a typed owned list
/// for a request field. It never constructs or invokes the selected shapes.
pub(super) struct RegisteredShapePickerTask {
    pub destination: SlotId,
    pub field: usize,
    handle: JoinHandle<eyre::Result<Option<Vec<RegisteredShape>>>>,
    cancellation: CancellationToken,
}

impl RegisteredShapePickerTask {
    pub(super) fn spawn(
        destination: SlotId,
        field: usize,
        cancellation: &CancellationScope,
    ) -> Result<Self, String> {
        let coordinator = TerminalCoordinator::try_current()
            .ok_or_else(|| "Shape selection requires the terminal coordinator".to_owned())?;
        let choices = RegisteredShape::all(known_functions(), known_things())
            .into_iter()
            .map(|shape| Choice {
                key: format!("{} · {}", shape.shape().type_identifier, shape),
                value: shape,
            })
            .collect::<Vec<_>>();
        let (cancellation, future) = cancellation.with_async_child_scope(move |scope| async move {
            match PickerTui::new()
                .set_header("Target shapes • Enter: add highlighted + checked • Tab: toggle • Ctrl-Enter: apply checked • Ctrl-D: clear")
                .set_auto_accept(false)
                .set_allow_empty_selection(true)
                .set_include_focused_on_enter(true)
                .cancel_on(scope.cancelled())
                .terminal_coordinator(coordinator)
                .pick_many(choices).await {
                    Ok(shapes) => Ok(Some(shapes)),
                    Err(PickError::Cancelled) => Ok(None),
                    Err(error) => Err(eyre::eyre!(error)),
                }
        });
        let handle = tokio::spawn(future);
        Ok(Self {
            destination,
            field,
            handle,
            cancellation,
        })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<Option<Vec<RegisteredShape>>, String> {
        self.handle
            .await
            .map_err(|error| format!("Target shape picker failed: {error}"))?
            .map_err(|error| error.to_string())
    }

    pub(super) async fn close(self) {
        self.cancellation.request_cancel("object explorer closed");
        if let Err(error) = self.finish().await {
            tracing::warn!(%error, "Target picker failed during shutdown");
        }
    }
}
