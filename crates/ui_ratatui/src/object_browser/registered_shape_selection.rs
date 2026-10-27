use super::app::BrowserMode;
use super::app::ObjectBrowserApp;
use super::controller::ObjectBrowserControllerError;
use super::registered_shape_picker_task::RegisteredShapePickerTask;
use crate::object_explorer::CardAddress;
use crate::object_explorer::SlotId;
use crate::object_explorer::ValueAddress;
use cloud_terrastodon_registry::RegisteredShape;
use facet::Facet;
use facet::Shape;

impl ObjectBrowserApp {
    pub(super) fn start_registered_shape_picker(
        &mut self,
        destination: SlotId,
        field: usize,
        shape: &'static Shape,
    ) -> Result<bool, ObjectBrowserControllerError> {
        if !shape.is_shape(Vec::<RegisteredShape>::SHAPE) {
            return Ok(false);
        }
        let Some(cancellation) = &self.cancellation else {
            self.status = "Target shape selection requires an active explorer session.".into();
            self.mode = BrowserMode::Pool;
            return Ok(true);
        };
        match RegisteredShapePickerTask::spawn(destination, field, cancellation) {
            Ok(task) => {
                self.registered_shape_picker = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status =
                    "Choose target shapes; Enter includes the highlighted search result.".into();
            }
            Err(message) => {
                self.mode = BrowserMode::Pool;
                self.status = message;
            }
        }
        Ok(true)
    }

    pub(super) async fn finish_registered_shape_picker(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .registered_shape_picker
            .as_ref()
            .is_some_and(RegisteredShapePickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .registered_shape_picker
            .take()
            .expect("finished picker");
        let destination = task.destination;
        let field = task.field;
        self.mode = BrowserMode::Pool;
        match task.finish().await {
            Ok(Some(shapes)) => {
                let count = shapes.len();
                let transition = self
                    .controller
                    .set_field_inline(destination, field, shapes)
                    .await?;
                self.controller
                    .focus(CardAddress::Value(ValueAddress::root(destination)))?;
                self.active_root = Some(
                    self.controller
                        .inspect_root(destination, self.max_relationship_rows)
                        .await?,
                );
                self.focus_after_builder_transition(destination, transition)?;
                self.status = format!("Set {count} target shapes on slot {destination}.");
            }
            Ok(None) => self.status = "Target shape selection cancelled.".into(),
            Err(error) => self.status = error,
        }
        Ok(())
    }
}
