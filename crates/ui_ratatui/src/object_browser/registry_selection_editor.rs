use super::app::BrowserMode;
use super::app::ObjectBrowserApp;
use super::controller::ObjectBrowserControllerError;
use super::registry_selection::RegistrySelection;
use super::registry_selection_picker_task::RegistrySelectionPickerTask;
use crate::object_explorer::CardAddress;
use crate::object_explorer::SlotId;
use crate::object_explorer::ValueAddress;
use facet::Shape;

impl ObjectBrowserApp {
    pub(super) fn start_registry_selection_picker(
        &mut self,
        destination: SlotId,
        field: usize,
        shape: &'static Shape,
    ) -> Result<bool, ObjectBrowserControllerError> {
        if !RegistrySelection::supports(shape) {
            return Ok(false);
        }
        let Some(cancellation) = &self.cancellation else {
            self.status = "Registry selection requires an active explorer session.".into();
            self.mode = BrowserMode::Pool;
            return Ok(true);
        };
        match RegistrySelectionPickerTask::spawn(destination, field, shape, cancellation) {
            Ok(task) => {
                self.registry_selection_picker = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = "Choose the registry identities for this field.".into();
            }
            Err(message) => {
                tracing::error!(error = %message, "Could not open registry selection picker");
                self.mode = BrowserMode::Pool;
                self.status = "Could not open registry picker; scroll up for full error.".into();
            }
        }
        Ok(true)
    }

    pub(super) async fn finish_registry_selection_picker(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .registry_selection_picker
            .as_ref()
            .is_some_and(RegistrySelectionPickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .registry_selection_picker
            .take()
            .expect("finished picker");
        let destination = task.destination;
        let field = task.field;
        self.mode = BrowserMode::Pool;
        match task.finish().await {
            Ok(Some(selection)) => {
                let count = selection.len();
                let description = selection.description();
                let transition = match selection {
                    RegistrySelection::Shapes(shapes) => {
                        self.controller
                            .set_field_inline(destination, field, shapes)
                            .await?
                    }
                    RegistrySelection::Functions(functions) => {
                        self.controller
                            .set_field_inline(destination, field, functions)
                            .await?
                    }
                };
                self.controller
                    .focus(CardAddress::Value(ValueAddress::root(destination)))?;
                self.active_root = Some(
                    self.controller
                        .inspect_root(destination, self.max_relationship_rows)
                        .await?,
                );
                self.focus_after_builder_transition(destination, transition)?;
                self.status = format!("Set {count} {description} on slot {destination}.");
            }
            Ok(None) => self.status = "Registry selection cancelled.".into(),
            Err(error) => {
                tracing::error!(error = %error, "Registry selection picker failed");
                self.status = "Registry picker failed; scroll up for full error.".into();
            }
        }
        Ok(())
    }
}
