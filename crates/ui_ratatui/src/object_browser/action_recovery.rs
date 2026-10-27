use super::app::BrowserMode;
use super::app::ObjectBrowserApp;
use super::controller::ObjectBrowserControllerError;
use crate::object_explorer::CardAddress;
use crate::object_explorer::ExplorerHandleError;
use crate::object_explorer::ValueAddress;

impl ObjectBrowserApp {
    /// Handle a rejected action after its command has returned and a nested
    /// picker, if any, has relinquished terminal ownership. Channel failures,
    /// query-session errors, and terminal/UI invariants still stop the session.
    pub(super) fn recover_action_error(
        &mut self,
        error: ObjectBrowserControllerError,
    ) -> Result<(), ObjectBrowserControllerError> {
        let ObjectBrowserControllerError::Request(ExplorerHandleError::Rejected(message)) = error
        else {
            return Err(error);
        };

        if let Some(link) = self.pending_link.take() {
            self.controller
                .focus(CardAddress::Value(ValueAddress::root(link.destination)))?;
        }
        // The engine owns rollback and failed-slot lifecycle. In particular a
        // rejected finalization may have produced a Failed slot; discard the
        // old Building snapshot rather than pretending construction succeeded.
        self.active_root = None;
        self.link_action_picker = None;
        self.value_picker = None;
        self.variant_picker = None;
        self.text_editor = None;
        self.breadcrumb_value_editor = None;
        self.tab_name_editor = None;
        self.mode = BrowserMode::Pool;
        self.status = format!("Action rejected: {message}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_browser::app::PendingFieldLink;
    use crate::object_browser::pickers::LinkActionPicker;
    use crate::object_explorer::ArenaQueryContext;
    use crate::object_explorer::ExplorerEngine;
    use crate::object_explorer::FieldBindingSnapshot;
    use crate::object_explorer::OwnedValuePacket;
    use crate::object_explorer::RootLifecycleSnapshot;
    use facet::Facet;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::Event;
    use ratatui::crossterm::event::KeyCode;
    use ratatui::crossterm::event::KeyEvent;
    use ratatui::crossterm::event::KeyModifiers;

    #[derive(Facet)]
    struct Destination {
        value: String,
    }

    #[tokio::test]
    async fn rejected_pending_and_selected_links_leave_the_browser_usable() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let handle = context.engine_handle();
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let (destination, _) = app
                .controller
                .create_builder(Destination::SHAPE)
                .await
                .unwrap();
            let source = handle
                .insert_ready(OwnedValuePacket::new(String::from("stale")))
                .await
                .unwrap();
            app.controller
                .focus(CardAddress::Value(ValueAddress::root(source)))
                .unwrap();
            app.active_root = Some(app.controller.inspect_root(source, 12).await.unwrap());

            // A pending field choice can become invalid between construction and
            // the tick that opens its transfer picker (for example after a variant change).
            app.pending_link = Some(PendingFieldLink {
                destination,
                field: usize::MAX,
                source: ValueAddress::root(source),
            });
            let error = app.tick().await.expect_err("stale field is rejected");
            app.recover_action_error(error).unwrap();
            assert_eq!(app.mode, BrowserMode::Pool);
            assert!(app.pending_link.is_none());
            assert!(app.status.starts_with("Action rejected:"));
            app.tick().await.unwrap();

            // The ordinary picker offered Move while the source was ready.
            // Deleting it before Enter exercises rejection after that preview.
            let actions = app
                .controller
                .inspect_field_candidate(destination, 0, ValueAddress::root(source))
                .await
                .unwrap();
            app.link_action_picker = Some(LinkActionPicker::new(actions));
            app.pending_link = Some(PendingFieldLink {
                destination,
                field: 0,
                source: ValueAddress::root(source),
            });
            app.mode = BrowserMode::LinkAction;
            handle.delete(source).await.unwrap();
            let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
            let error = app
                .handle_event(&enter)
                .await
                .expect_err("the selected source disappeared");
            app.recover_action_error(error).unwrap();
            assert!(!app.should_quit);
            assert_eq!(app.mode, BrowserMode::Pool);
            assert!(app.link_action_picker.is_none());
            assert!(app.pending_link.is_none());
            assert!(app.status.starts_with("Action rejected:"));
            app.tick().await.unwrap();
            let destination_state = app.controller.inspect_root(destination, 12).await.unwrap();
            assert_eq!(
                destination_state.lifecycle(),
                &RootLifecycleSnapshot::Building
            );
            assert_eq!(
                destination_state.builder().unwrap().fields()[0].binding(),
                &FieldBindingSnapshot::Unset
            );

            let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            assert!(terminal.backend().to_string().contains("Action rejected:"));

            let replacement = handle
                .insert_ready(OwnedValuePacket::new(String::from("recovered")))
                .await
                .unwrap();
            app.pending_link = Some(PendingFieldLink {
                destination,
                field: 0,
                source: ValueAddress::root(replacement),
            });
            app.open_pending_link_actions().await.unwrap();
            assert_eq!(app.mode, BrowserMode::LinkAction);
            app.handle_event(&enter).await.unwrap();
            assert_eq!(
                app.active_root.as_ref().unwrap().lifecycle(),
                &RootLifecycleSnapshot::Ready
            );
            app.tick().await.unwrap();
            app.close().await.unwrap();
            drop(handle);
            destination
        };
        let (engine, destination) = tokio::join!(engine.run(inbox), client);
        assert_eq!(
            engine
                .arena()
                .ready_value(destination)
                .unwrap()
                .peek()
                .get::<Destination>()
                .unwrap()
                .value,
            "recovered"
        );
    }

    #[tokio::test]
    async fn a_disconnected_engine_is_not_recovered_as_a_user_action() {
        tokio::task::LocalSet::new()
            .run_until(async {
                let (context, inbox) = ArenaQueryContext::channel(8);
                let task = tokio::task::spawn_local(ExplorerEngine::empty().run(inbox));
                let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
                task.abort();
                assert!(
                    task.await
                        .err()
                        .expect("engine task was aborted")
                        .is_cancelled()
                );
                let error = app.tick().await.expect_err("the engine is disconnected");
                let error = app
                    .recover_action_error(error)
                    .expect_err("channel failure remains fatal");
                assert!(matches!(
                    error,
                    ObjectBrowserControllerError::Request(ExplorerHandleError::EngineStopped)
                ));
                drop(app);
            })
            .await;
    }
}
