use super::registry_selection::RegistrySelection;
use crate::object_explorer::SlotId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_registry::RegisteredFunction;
use cloud_terrastodon_registry::RegisteredShape;
use cloud_terrastodon_registry::known_functions;
use cloud_terrastodon_registry::known_things;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use facet::Facet;
use facet::Shape;
use tokio::task::JoinHandle;

/// A terminal selection of registry identities, producing a typed owned list
/// for a request field. It never constructs or invokes the selected shapes.
pub(super) struct RegistrySelectionPickerTask {
    pub destination: SlotId,
    pub field: usize,
    handle: JoinHandle<eyre::Result<Option<RegistrySelection>>>,
    cancellation: CancellationToken,
}

impl RegistrySelectionPickerTask {
    pub(super) fn spawn(
        destination: SlotId,
        field: usize,
        shape: &'static Shape,
        cancellation: &CancellationScope,
    ) -> Result<Self, String> {
        let coordinator = TerminalCoordinator::try_current()
            .ok_or_else(|| "Registry selection requires the terminal coordinator".to_owned())?;
        let (cancellation, future) = cancellation.with_async_child_scope(move |scope| async move {
            if shape.is_shape(Vec::<RegisteredShape>::SHAPE) {
                let choices = RegisteredShape::all(known_functions(), known_things())
                    .into_iter()
                    .map(|shape| Choice {
                        key: format!("{} · {}", shape.shape().type_identifier, shape),
                        value: shape,
                    })
                    .collect::<Vec<_>>();
                match PickerTui::new()
                    .set_header("Shapes (seeds or targets) • Enter: add highlighted + checked • Tab: toggle • Ctrl-Enter: apply checked • Ctrl-D: clear")
                    .set_auto_accept(false)
                    .set_allow_empty_selection(true)
                    .set_include_focused_on_enter(true)
                    .cancel_on(scope.cancelled())
                    .terminal_coordinator(coordinator)
                    .pick_many(choices)
                    .await {
                        Ok(shapes) => Ok(Some(RegistrySelection::Shapes(shapes))),
                        Err(PickError::Cancelled) => Ok(None),
                        Err(error) => Err(eyre::eyre!(error)),
                    }
            } else {
                let mut choices = known_functions()
                    .iter()
                    .map(RegisteredFunction::from_function)
                    .map(|function| Choice {
                        key: function.to_string(),
                        value: function,
                    })
                    .collect::<Vec<_>>();
                choices.sort_by(|left, right| left.key.cmp(&right.key));
                let selected = choices.iter().map(|choice| choice.key.clone()).collect::<Vec<_>>();
                match PickerTui::new()
                    .set_header("Functions • All initially selected • Tab: toggle • Enter: apply checked • Ctrl-D: clear")
                    .set_auto_accept(false)
                    .set_allow_empty_selection(true)
                    .set_initial_selected(selected)
                    .cancel_on(scope.cancelled())
                    .terminal_coordinator(coordinator)
                    .pick_many(choices)
                    .await {
                        Ok(functions) => Ok(Some(RegistrySelection::Functions(functions))),
                        Err(PickError::Cancelled) => Ok(None),
                        Err(error) => Err(eyre::eyre!(error)),
                    }
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

    pub(super) async fn finish(self) -> Result<Option<RegistrySelection>, String> {
        self.handle
            .await
            .map_err(|error| format!("Registry selection picker failed: {error}"))?
            .map_err(|error| error.to_string())
    }

    pub(super) async fn close(self) {
        self.cancellation.request_cancel("object explorer closed");
        if let Err(error) = self.finish().await {
            tracing::warn!(%error, "Registry selection picker failed during shutdown");
        }
    }
}
