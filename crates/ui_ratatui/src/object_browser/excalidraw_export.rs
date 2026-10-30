use super::app::ObjectBrowserApp;
use super::controller::ObjectBrowserControllerError;
use crate::object_explorer::CardAddress;
use crate::object_explorer::ValueAddress;
use cloud_terrastodon_excalidraw::ExcalidrawScene;

impl ObjectBrowserApp {
    pub(super) fn render_focused_slot(&mut self) {
        let CardAddress::Value(address) = self.selected_address() else {
            self.status = "Select an object slot before pressing Ctrl-O.".into();
            return;
        };
        let context = self.controller.query_context();
        let Some(renderer) = &mut self.excalidraw else {
            self.status = "This object browser has no Excalidraw destination.".into();
            return;
        };
        let result = renderer.start(move |scope| async move {
            let object = context
                .engine_handle()
                .inspect_diagram(ValueAddress::root(address.root_id()), 256)
                .await?;
            scope.bail_if_cancelled()?;
            ExcalidrawScene::from_objects(
                &format!("Object explorer · slot {}", address.root_id()),
                &[object],
            )
        });
        self.status = match result {
            Ok(path) => format!(
                "Rendering slot to \"{}\"; waiting if the file is locked…",
                path.display()
            ),
            Err(error) => {
                tracing::error!(error = %format_args!("{error:#}"), "Could not render slot");
                "Could not render slot; scroll up for full error.".into()
            }
        };
    }

    pub(super) async fn render_current_tab(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let breadcrumbs = self.controller.active_breadcrumbs().await?;
        let title = format!(
            "Object explorer · {}",
            self.controller.active_tab_header().name()
        );
        let context = self.controller.query_context();
        let Some(renderer) = &mut self.excalidraw else {
            self.status = "This object browser has no Excalidraw destination.".into();
            return Ok(());
        };
        let result = renderer.start(move |scope| async move {
            let session = context.open_export(breadcrumbs).await?;
            let mut objects = Vec::new();
            loop {
                scope.bail_if_cancelled()?;
                let batch = session.next_diagram_batch(256, 32, 256).await?;
                objects.extend(batch.objects);
                if batch.complete {
                    break;
                }
                tokio::task::yield_now().await;
            }
            session.complete().await?;
            scope.bail_if_cancelled()?;
            ExcalidrawScene::from_objects(&title, &objects)
        });
        self.status = match result {
            Ok(path) => format!(
                "Rendering tab to \"{}\"; waiting if the file is locked…",
                path.display()
            ),
            Err(error) => {
                tracing::error!(error = %format_args!("{error:#}"), "Could not render tab");
                "Could not render tab; scroll up for full error.".into()
            }
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "excalidraw_export_tests.rs"]
mod tests;
