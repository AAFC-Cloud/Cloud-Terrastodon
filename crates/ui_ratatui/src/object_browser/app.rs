use super::breadcrumb_bar_focus::BreadcrumbBarFocus;
use super::breadcrumb_filter_field_picker_task::BreadcrumbFilterFieldPickerTask;
use super::breadcrumb_menu_task::BreadcrumbMenuTask;
use super::breadcrumb_picker_task::BreadcrumbPickerTask;
use super::breadcrumb_value_picker_task::BreadcrumbValuePickerTask;
use super::controller::ObjectBrowserController;
use super::controller::ObjectBrowserControllerError;
use super::excalidraw_render_session::ExcalidrawRenderSession;
use super::link_action_picker_task::LinkActionPickerTask;
use super::pickers::FieldValuePicker;
use super::pickers::LinkActionPicker;
use super::pickers::VariantPicker;
use super::registry_selection_picker_task::RegistrySelectionPickerTask;
use super::render::CardLayoutAxis;
use super::row_search::RowSearchState;
use super::shape_picker_task::ShapePickerTask;
use super::value_picker_task::ValuePickerTask;
use super::variant_picker_task::VariantPickerTask;
use crate::object_explorer::ArenaQueryContext;
use crate::object_explorer::BuilderKindSnapshot;
use crate::object_explorer::CardAddress;
use crate::object_explorer::CardSnapshot;
use crate::object_explorer::InvocationEventState;
use crate::object_explorer::ProductionBatch;
use crate::object_explorer::ProductionJobState;
use crate::object_explorer::QueryProgressState;
use crate::object_explorer::RootLifecycleSnapshot;
use crate::object_explorer::RootSnapshot;
use crate::object_explorer::SlotId;
use crate::object_explorer::ValueAddress;
use cloud_terrastodon_command::CancellationScope;
use std::num::NonZeroUsize;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BrowserMode {
    Pool,
    RowSearch,
    Variant,
    Value,
    LinkAction,
    Text,
    BreadcrumbValue,
    NestedPicker,
    TabName,
}

#[derive(Clone, Debug)]
pub(super) struct PendingFieldLink {
    pub(super) destination: SlotId,
    pub(super) field: usize,
    pub(super) source: ValueAddress,
}

pub(super) struct TextEditorState {
    pub(super) slot: SlotId,
    pub(super) shape: &'static facet::Shape,
    pub(super) text: String,
}

pub(super) struct BreadcrumbValueEditorState {
    pub(super) edit_index: Option<usize>,
    pub(super) field_shape: String,
    pub(super) field_name: String,
    pub(super) operator: crate::object_explorer::ValueFilterOperator,
    pub(super) text: String,
}

/// Thin Ratatui adapter over ObjectBrowserController.
///
/// The app owns no RuntimeValue, Arena, builder, or flattened object pool. Its
/// retained data is limited to UI state and bounded engine snapshots.
pub(crate) struct ObjectBrowserApp {
    pub(super) controller: ObjectBrowserController,
    pub(super) should_quit: bool,
    pub(super) recent_escape_presses: Vec<Instant>,
    pub(super) mode: BrowserMode,
    pub(super) axis: CardLayoutAxis,
    pub(super) focused_card_fill: bool,
    pub(super) card_width: u16,
    pub(super) card_height: u16,
    pub(super) last_card_main_axis: u16,
    pub(super) max_cards: NonZeroUsize,
    pub(super) max_relationship_rows: usize,
    pub(super) active_root: Option<RootSnapshot>,
    pub(super) shape_picker_task: Option<ShapePickerTask>,
    pub(super) variant_picker: Option<VariantPicker>,
    pub(super) variant_picker_task: Option<VariantPickerTask>,
    pub(super) value_picker: Option<FieldValuePicker>,
    pub(super) value_picker_task: Option<ValuePickerTask>,
    pub(super) link_action_picker: Option<LinkActionPicker>,
    pub(super) link_action_picker_task: Option<LinkActionPickerTask>,
    pub(super) pending_link: Option<PendingFieldLink>,
    pub(super) text_editor: Option<TextEditorState>,
    pub(super) breadcrumb_focus: Option<BreadcrumbBarFocus>,
    pub(super) breadcrumb_menu_task: Option<BreadcrumbMenuTask>,
    pub(super) breadcrumb_filter_field_picker_task: Option<BreadcrumbFilterFieldPickerTask>,
    pub(super) breadcrumb_value_editor: Option<BreadcrumbValueEditorState>,
    pub(super) breadcrumb_picker_task: Option<BreadcrumbPickerTask>,
    pub(super) breadcrumb_value_picker_task: Option<BreadcrumbValuePickerTask>,
    pub(super) row_search: Option<RowSearchState>,
    pub(super) tab_name_editor: Option<String>,
    pub(super) production_jobs_active: bool,
    pub(super) status: String,
    pub(super) excalidraw: Option<ExcalidrawRenderSession>,
    pub(super) registry_selection_picker: Option<RegistrySelectionPickerTask>,
    pub(super) cancellation: Option<CancellationScope>,
}

impl ObjectBrowserApp {
    pub(super) const FRAME_WORK: usize = 256;
    pub(super) const MIN_CARD_WIDTH: u16 = 34;
    pub(super) const MIN_CARD_HEIGHT: u16 = 7;

    pub(crate) async fn bootstrap(
        context: ArenaQueryContext,
    ) -> Result<Self, ObjectBrowserControllerError> {
        let mut controller = ObjectBrowserController::bootstrap(context).await?;
        controller.select(CardAddress::NewSlot)?;
        Ok(Self {
            controller,
            should_quit: false,
            recent_escape_presses: Vec::new(),
            mode: BrowserMode::Pool,
            axis: CardLayoutAxis::Horizontal,
            focused_card_fill: false,
            card_width: 38,
            card_height: 10,
            last_card_main_axis: 0,
            max_cards: NonZeroUsize::MIN.saturating_add(4),
            max_relationship_rows: 12,
            active_root: None,
            shape_picker_task: None,
            variant_picker: None,
            variant_picker_task: None,
            value_picker: None,
            value_picker_task: None,
            link_action_picker: None,
            link_action_picker_task: None,
            pending_link: None,
            text_editor: None,
            breadcrumb_focus: None,
            breadcrumb_menu_task: None,
            breadcrumb_filter_field_picker_task: None,
            breadcrumb_value_editor: None,
            breadcrumb_picker_task: None,
            breadcrumb_value_picker_task: None,
            row_search: None,
            tab_name_editor: None,
            production_jobs_active: false,
            status: "Left/Right: cards | Up/Down: rows | Ctrl-O: draw slot | Ctrl-U: draw tab | Shift+End: new object | Esc: back / triple-Esc: exit".to_owned(),
            excalidraw: None,
            registry_selection_picker: None,
            cancellation: None,
        })
    }

    pub(super) async fn tick(&mut self) -> Result<(), ObjectBrowserControllerError> {
        if let Some(renderer) = &mut self.excalidraw {
            if let Some(result) = renderer.take_result().await {
                self.status = match result {
                    Ok(path) => format!(
                        "Rendered Excalidraw: \"{}\" (Ctrl-O: slot, Ctrl-U: tab)",
                        path.display()
                    ),
                    Err(error) => {
                        tracing::error!(error = %format_args!("{error:#}"), "Excalidraw export failed");
                        "Excalidraw export failed; scroll up for full error.".to_owned()
                    }
                };
            }
            if renderer.is_rendering() {
                return Ok(());
            }
        }
        if self.controller.query_context().is_export_active() {
            return Ok(());
        }
        if self.mode == BrowserMode::NestedPicker {
            if self.registry_selection_picker.is_some() {
                self.finish_registry_selection_picker().await?;
            } else if self.breadcrumb_menu_task.is_some() {
                self.finish_breadcrumb_menu_task().await?;
            } else if self.breadcrumb_filter_field_picker_task.is_some() {
                self.finish_breadcrumb_filter_field_picker_task().await?;
            } else if self.shape_picker_task.is_some() {
                self.finish_shape_picker_task().await?;
            } else if self.variant_picker_task.is_some() {
                self.finish_variant_picker_task().await?;
            } else if self.value_picker_task.is_some() {
                self.finish_value_picker_task().await?;
            } else if self.link_action_picker_task.is_some() {
                self.finish_link_action_picker_task().await?;
            } else if self.breadcrumb_value_picker_task.is_some() {
                self.finish_breadcrumb_value_picker_task().await?;
            } else {
                self.finish_breadcrumb_picker_task().await?;
            }
            return Ok(());
        }
        let events = self.controller.poll_invocations().await?;
        let mut latest_failure = None;
        // Polling drains completed events. Log every failure once, even when a
        // later success is the most recent event in this batch.
        for event in &events {
            if let InvocationEventState::Failed(message) = &event.state {
                tracing::error!(
                    input_slot = %event.input,
                    output_slot = %event.output,
                    error = %message,
                    "Object invocation failed"
                );
                latest_failure = Some(event);
            }
        }
        if let Some(event) = latest_failure.or_else(|| events.last()) {
            self.status = match &event.state {
                InvocationEventState::Failed(_) => format!(
                    "Invocation slot {} failed; scroll up for full error.",
                    event.output
                ),
                state => format!("Invocation output slot {} is {state:?}.", event.output),
            };
        }
        if self.production_jobs_active {
            let batch = self
                .controller
                .advance_productions(Self::FRAME_WORK)
                .await?;
            self.apply_production_batch(&batch);
        }

        if self.mode == BrowserMode::Value {
            self.refresh_value_candidates().await?;
            return Ok(());
        }

        if self.mode == BrowserMode::Pool {
            if let Some(slot) = self.active_root.as_ref().map(RootSnapshot::slot) {
                let snapshot = self
                    .controller
                    .inspect_root(slot, self.max_relationship_rows)
                    .await?;
                let became_ready = matches!(snapshot.lifecycle(), RootLifecycleSnapshot::Ready);
                self.active_root = Some(snapshot);
                if became_ready
                    && self
                        .pending_link
                        .as_ref()
                        .is_some_and(|link| link.source == ValueAddress::root(slot))
                {
                    self.open_pending_link_actions().await?;
                }
            }
            self.controller
                .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
                .await?;
        }
        Ok(())
    }

    pub(super) async fn refresh_value_candidates(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        let progress = self
            .controller
            .fill_value_candidates(None, Self::FRAME_WORK, NonZeroUsize::MIN.saturating_add(7))
            .await?;
        match progress.into_state() {
            QueryProgressState::Ready(window) => {
                if let Some(picker) = self.value_picker.as_mut() {
                    picker.replace_window(window);
                }
                self.start_value_picker_task();
            }
            QueryProgressState::Pending => self.status = "Scanning compatible values…".to_owned(),
            QueryProgressState::Complete => {
                self.start_value_picker_task();
            }
            QueryProgressState::Cancelled | QueryProgressState::Stale => {
                self.mode = BrowserMode::Pool;
            }
        }
        Ok(())
    }

    pub(super) async fn open_pending_link_actions(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        let Some(link) = self.pending_link.clone() else {
            return Ok(());
        };
        let actions = self
            .controller
            .inspect_field_candidate(link.destination, link.field, link.source)
            .await?;
        self.link_action_picker = Some(LinkActionPicker::new(actions));
        if !self.start_link_action_picker_task() {
            self.mode = BrowserMode::LinkAction;
        }
        Ok(())
    }

    pub(super) fn selected_card(&self) -> Option<&CardSnapshot> {
        let selection = self.controller.active_state()?.selection();
        self.active_root
            .as_ref()
            .map(RootSnapshot::card)
            .filter(|card| card.address() == selection)
            .or_else(|| {
                self.controller
                    .window()?
                    .cards()
                    .iter()
                    .find(|card| card.address() == selection)
            })
    }

    pub(super) fn scalar_editor_for_root(&mut self, slot: SlotId) {
        let Some(root) = &self.active_root else {
            return;
        };
        let Some(builder) = root.builder() else {
            return;
        };
        let Some(shape) = builder.shape() else {
            return;
        };
        if matches!(builder.kind(), BuilderKindSnapshot::Scalar { .. }) {
            self.text_editor = Some(TextEditorState {
                slot,
                shape,
                text: String::new(),
            });
            self.mode = BrowserMode::Text;
        }
    }

    pub(super) fn selected_address(&self) -> CardAddress {
        self.controller
            .active_state()
            .map(|state| state.selection().clone())
            .unwrap_or(CardAddress::NewSlot)
    }

    pub(super) fn apply_production_batch(&mut self, batch: &ProductionBatch) {
        self.production_jobs_active = batch.active_jobs() != 0;
        let mut latest_failure = None;
        // Terminal producer updates are removed from the active jobs, so their
        // full diagnostics are emitted once rather than on every redraw.
        for update in batch.updates() {
            if let ProductionJobState::Failed { message } = update.state() {
                tracing::error!(
                    job = %update.job(),
                    destination_slot = %update.destination(),
                    field = update.field(),
                    error = %message,
                    "Object producer failed"
                );
                latest_failure = Some(update);
            }
        }
        let Some(update) = latest_failure.or_else(|| batch.updates().last()) else {
            return;
        };
        self.status = match update.state() {
            ProductionJobState::Running { latest_root } => latest_root.map_or_else(
                || format!("Producer job {} is waiting.", update.job()),
                |root| {
                    format!(
                        "Producer job {} advanced through slot {root} ({} work units).",
                        update.job(),
                        batch.work_spent()
                    )
                },
            ),
            ProductionJobState::Complete {
                output,
                destination_transition,
            } => format!(
                "Producer job {} moved output slot {output} into slot {} field {} ({destination_transition:?}).",
                update.job(),
                update.destination(),
                update.field()
            ),
            ProductionJobState::Failed { .. } => {
                format!(
                    "Producer job {} failed; scroll up for full error.",
                    update.job()
                )
            }
        };
    }

    pub(crate) async fn close(mut self) -> Result<(), ObjectBrowserControllerError> {
        if let Some(picker) = self.registry_selection_picker.take() {
            picker.close().await;
        }
        if let Some(renderer) = self.excalidraw.take() {
            renderer.close().await;
        }
        self.controller.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::ExplorerEngine;
    use facet::Facet;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct AppBuilderThing {
        name: String,
    }

    #[tokio::test]
    async fn new_app_bootstrap_and_first_tick_use_only_engine_snapshots() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            assert_eq!(app.selected_address(), CardAddress::NewSlot);
            app.tick().await.unwrap();
            assert!(app.controller.window().is_some());
            assert!(app.active_root.is_none());
            let backend = TestBackend::new(120, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains("+ create object"));
            assert!(rendered.contains("vvvvv"));
            assert!(rendered.contains("^^^^^"));
            app.controller.close().await.unwrap();
        };
        let (engine, ()) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
        assert_eq!(engine.arena().allocated_slot_count(), 1);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn active_builder_scrolls_into_the_bounded_pool_window() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client =
            async move {
                let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
                app.tick().await.unwrap();
                let (slot, transition) = app
                    .controller
                    .create_builder(AppBuilderThing::SHAPE)
                    .await
                    .unwrap();
                assert_eq!(
                    transition,
                    crate::object_explorer::BuilderTransition::Building
                );
                app.controller
                    .focus(CardAddress::Value(ValueAddress::root(slot)))
                    .unwrap();
                app.active_root = Some(
                    app.controller
                        .inspect_root(slot, app.max_relationship_rows)
                        .await
                        .unwrap(),
                );
                app.tick().await.unwrap();

                let window = app.controller.window().unwrap();
                assert!(window.cards().len() > 1);
                assert!(window.cards().iter().any(|card| {
                    card.address() == &CardAddress::Value(ValueAddress::root(slot))
                }));
                let companion = window
                    .cards()
                    .iter()
                    .rev()
                    .find(|card| card.address() != &CardAddress::Value(ValueAddress::root(slot)))
                    .unwrap()
                    .address()
                    .clone();

                let backend = TestBackend::new(120, 24);
                let mut terminal = Terminal::new(backend).unwrap();
                terminal.draw(|frame| app.draw(frame)).unwrap();
                let rendered = terminal.backend().to_string();
                assert!(rendered.contains(&format!("slot {slot} [owned]")));
                assert!(rendered.contains("type String"));
                assert!(rendered.contains("field \"name\""));
                assert!(rendered.contains("value unset"));
                assert!(rendered.contains("vvvvv"));
                assert!(rendered.contains("^^^^^"));
                let CardAddress::Value(companion) = companion else {
                    panic!("query cards have value addresses");
                };
                assert!(rendered.contains(&companion.to_string()));
                app.controller.close().await.unwrap();
            };

        let (_engine, ()) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
    }
}
