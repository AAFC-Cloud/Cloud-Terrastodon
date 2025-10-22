use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::PrincipalCollection;
use cloud_terrastodon_azure::prelude::RoleAssignmentId;
use cloud_terrastodon_azure::prelude::RoleDefinitionsAndAssignments;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_command::app_work::AppWorkState;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eyre::Result;
use itertools::Itertools;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::mem;
use std::time::Duration;
use strum::VariantArray;

/// Result of running the role assignment picker application.
pub enum RoleAssignmentPickerAppResult {
    Cancelled,
    Some {
        chosen_role_assignment_ids: Vec<RoleAssignmentId>,
        role_definitions_and_assignments: RoleDefinitionsAndAssignments,
        principals: PrincipalCollection,
    },
}

impl fmt::Debug for RoleAssignmentPickerAppResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoleAssignmentPickerAppResult::Cancelled => f.debug_struct("Cancelled").finish(),
            RoleAssignmentPickerAppResult::Some {
                chosen_role_assignment_ids,
                ..
            } => f
                .debug_struct("Some")
                .field("chosen_role_assignment_ids", chosen_role_assignment_ids)
                .finish(),
        }
    }
}

/// Entrypoint for the interactive role assignment picker.
pub struct RoleAssignmentPickerApp {}

#[derive(strum::VariantArray)]
pub enum TableColumn {
    Selected,
    PrincipalDisplayName,
    PrincipalType,
    RoleDisplayName,
    Scope,
}
impl TableColumn {
    pub fn header(&self) -> Cell<'static> {
        match self {
            TableColumn::Selected => Cell::from("Sel"),
            TableColumn::PrincipalDisplayName => Cell::from("Principal"),
            TableColumn::PrincipalType => Cell::from("Principal Type"),
            TableColumn::RoleDisplayName => Cell::from("Role"),
            TableColumn::Scope => Cell::from("Scope"),
        }
    }

    pub fn constraint(&self) -> Constraint {
        match self {
            TableColumn::Selected => Constraint::Length(4),
            TableColumn::PrincipalDisplayName => Constraint::Percentage(25),
            TableColumn::PrincipalType => Constraint::Percentage(25),
            TableColumn::RoleDisplayName => Constraint::Length(14),
            TableColumn::Scope => Constraint::Fill(1),
        }
    }
}

impl RoleAssignmentPickerApp {
    /// Creates a new app instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Runs the interactive picker. Returns the user's selection or cancellation.
    pub async fn run(self) -> Result<RoleAssignmentPickerAppResult> {
        // Declare variables
        let mut rbac: RefCell<Loadable<RoleDefinitionsAndAssignments>> = Default::default();
        let mut rbac_work: AppWorkState<_> = Default::default();

        let mut principals: RefCell<Loadable<PrincipalCollection>> = Default::default();
        let mut principals_work: AppWorkState<_> = Default::default();

        let mut table_state: TableState = Default::default();
        let mut rows: Option<Vec<Row<'_>>> = Default::default();
        let mut status_message: String = "Loading role assignments and principals...".to_string();

        // Used for page-up and page-down calculations
        let mut last_table_height: u16 = Default::default();

        let mut selected_role_assignments: FxHashSet<RoleAssignmentId> = Default::default();

        // Dispatch background work
        LoadableWorkBuilder::new()
            .description("fetch_all_role_definitions_and_assignments")
            .setter(
                |state: &mut RefCell<Loadable<RoleDefinitionsAndAssignments>>, value| {
                    *state.borrow_mut() = value;
                },
            )
            .work(fetch_all_role_definitions_and_assignments())
            .build()?
            .enqueue(&rbac_work, &mut rbac)?;

        LoadableWorkBuilder::new()
            .description("fetch_all_principals")
            .setter(
                |state: &mut RefCell<Loadable<PrincipalCollection>>, value| {
                    *state.borrow_mut() = value;
                },
            )
            .work(fetch_all_principals())
            .build()?
            .enqueue(&principals_work, &mut principals)?;

        // Construct header
        let header = Row::new(
            TableColumn::VARIANTS
                .iter()
                .map(|col| col.header())
                .collect::<Vec<_>>(),
        )
        .style(Style::default().add_modifier(Modifier::BOLD));

        // Initialize terminal
        let mut terminal = ratatui::init();
        terminal.clear()?;

        let result = 'outer: loop {
            // 1. Handle background work completion
            rbac_work.handle_messages(&mut rbac)?;
            principals_work.handle_messages(&mut principals)?;

            // 2. Handle keyboard input
            while event::poll(Duration::from_millis(0))? {
                let result = match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('c') => {
                                    Some(RoleAssignmentPickerAppResult::Cancelled)
                                }
                                KeyCode::Char('a') => {
                                    match (rows.as_ref(), rbac.borrow().as_loaded()) {
                                        (Some(rows), Some(rbac)) => {
                                            for (assignment, _definition) in
                                                rbac.iter_role_assignments()
                                            {
                                                selected_role_assignments
                                                    .insert(assignment.id.clone());
                                            }
                                            self.update_status_message(
                                                &rows,
                                                &selected_role_assignments,
                                                &mut status_message,
                                            );
                                        }
                                        _ => {}
                                    }

                                    None
                                }
                                // KeyCode::Char('d') => {
                                //     self.clear_selection();
                                //     None
                                // }
                                // KeyCode::Char('t') => {
                                //     self.invert_selection();
                                //     None
                                // }
                                _ => None,
                            }
                        } else {
                            match key.code {
                                KeyCode::Esc => Some(RoleAssignmentPickerAppResult::Cancelled),
                                KeyCode::Up => {
                                    table_state.select_previous();
                                    None
                                }
                                KeyCode::Down => {
                                    table_state.select_next();
                                    None
                                }
                                KeyCode::PageUp => {
                                    for _ in 0..last_table_height {
                                        table_state.select_previous();
                                    }
                                    None
                                }
                                KeyCode::PageDown => {
                                    for _ in 0..last_table_height {
                                        table_state.select_next();
                                    }
                                    None
                                }
                                KeyCode::Home => {
                                    table_state.select_first();
                                    None
                                }
                                KeyCode::End => {
                                    table_state.select_last();
                                    None
                                }
                                // KeyCode::Tab => {
                                //     let Some(selected_idx) = table_state.selected() else {
                                //         continue;
                                //     };
                                //     let Some(row) = rows.as_ref().map(|rows| rows.get(selected_idx)) else {
                                //         continue;
                                //     };

                                //     if !self
                                //         .selected_role_assignments
                                //         .insert(row.role_assignment_id.clone())
                                //     {
                                //         self.selected_role_assignments
                                //             .remove(&row.role_assignment_id);
                                //     }
                                //     self.update_status_message();
                                //     table_state.select_next();
                                //     None
                                // }
                                // KeyCode::Enter => {
                                //     if let Some(result) = self.build_success_result()? {
                                //         Some(result)
                                //     } else {
                                //         None
                                //     }
                                // }
                                _ => None,
                            }
                        }
                    }
                    _ => None,
                };
                if let Some(result) = result {
                    break 'outer result;
                }
            }

            // 3. Rebuild rows if data changed
            if rows.is_none() {
                let rbac = rbac.borrow();
                let principals = principals.borrow();
                match rbac.as_loaded() {
                    Some(rbac) => match principals.as_loaded() {
                        Some(principals) => {
                            let new_rows: Vec<Row<'_>> = rbac
                                .iter_role_assignments()
                                .map(|(assignment, definition)| {
                                    let (principal_display_name, principal_type) = principals
                                        .get(&assignment.principal_id)
                                        .map(|principal| match principal {
                                            Principal::User(user) => (
                                                Cow::Borrowed(user.user_principal_name.as_str()),
                                                "User",
                                            ),
                                            Principal::Group(group) => (
                                                Cow::Borrowed(group.display_name.as_str()),
                                                "Group",
                                            ),
                                            Principal::ServicePrincipal(sp) => (
                                                Cow::Borrowed(sp.display_name.as_str()),
                                                "Service Principal",
                                            ),
                                        })
                                        .unwrap_or_else(|| (Cow::Borrowed("Unknown"), "Unknown"));

                                    let is_selected =
                                        selected_role_assignments.contains(&assignment.id);
                                    let marker = if is_selected { "[x]" } else { "[ ]" };
                                    (
                                        is_selected,
                                        marker,
                                        principal_display_name,
                                        principal_type,
                                        definition.display_name.as_str(),
                                        assignment.scope.expanded_form(),
                                    )
                                })
                                .sorted_by(
                                    #[expect(unused)]
                                    |(
                                        left_is_selected,
                                        left_marker,
                                        left_principal_display_name,
                                        left_principal_type,
                                        left_role_definition_display_name,
                                        left_role_assignment_scope,
                                    ),
                                     (
                                        right_is_selected,
                                        right_marker,
                                        right_principal_display_name,
                                        right_principal_type,
                                        right_role_definition_display_name,
                                        right_role_assignment_scope,
                                    )| {
                                        left_role_assignment_scope
                                            .cmp(right_role_assignment_scope)
                                            .then_with(|| {
                                                left_principal_display_name
                                                    .cmp(right_principal_display_name)
                                            })
                                            .then_with(|| {
                                                left_role_definition_display_name
                                                    .cmp(right_role_definition_display_name)
                                            })
                                    },
                                )
                                .map(
                                    |(
                                        is_selected,
                                        marker,
                                        principal_display_name,
                                        principal_type,
                                        role_definition_display_name,
                                        role_assignment_scope,
                                    )| {
                                        let mut table_row = Row::new(vec![
                                            Cell::from(marker),
                                            Cell::from(principal_display_name),
                                            Cell::from(principal_type),
                                            Cell::from(role_definition_display_name),
                                            Cell::from(role_assignment_scope),
                                        ]);

                                        if is_selected {
                                            table_row =
                                                table_row.style(Style::default().fg(Color::Green));
                                        }
                                        table_row
                                    },
                                )
                                .collect();
                            rows = Some(new_rows);
                        }
                        None => {
                            status_message = "Loading principals...".to_string();
                        }
                    },
                    None => {
                        status_message = "Loading role assignments...".to_string();
                    }
                };

                if let Some(ref rows) = rows {
                    self.update_status_message(
                        &rows,
                        &selected_role_assignments,
                        &mut status_message,
                    );
                }
            }

            // 4. Draw the UI
            terminal.draw(|frame: &mut Frame<'_>| {
                let [status_area, table_area] =
                    Layout::vertical([Constraint::Length(3), Constraint::Min(1)])
                        .areas(frame.area());
                last_table_height = table_area.height.saturating_sub(2).max(1);

                let status_paragraph = Paragraph::new(status_message.as_str()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Role Assignment Picker"),
                );
                frame.render_widget(status_paragraph, status_area);

                if let Some(ref rows) = rows {
                    let table = {
                        Table::new(
                            rows.clone(), // ugly! https://github.com/ratatui/ratatui/issues/1004
                            TableColumn::VARIANTS
                                .iter()
                                .map(|col| col.constraint())
                                .collect::<Vec<_>>(),
                        )
                        .header(header.clone())
                        .block(Block::default().borders(Borders::ALL))
                        .highlight_symbol("▶ ")
                        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    };
                    frame.render_stateful_widget(table, table_area, &mut table_state);
                }
            })?;

            // 5. Throttle loop to avoid busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        };

        ratatui::restore();
        rbac_work.work_tracker.finish().await?;
        principals_work.work_tracker.finish().await?;
        Ok(result)
    }

    fn update_status_message(
        &self,
        rows: &Vec<Row<'_>>,
        selected_role_assignments: &FxHashSet<RoleAssignmentId>,
        status_message: &mut String,
    ) {
        if rows.is_empty() {
            *status_message = "No role assignments found.".to_string();
            return;
        }

        let selected = selected_role_assignments.len();
        let total = rows.len();
        *status_message = format!(
            "Loaded {total} role assignments | Selected {selected} | Esc to cancel, Enter to accept"
        );
    }

    // fn move_selection_up(&mut self, amount: usize) {
    //     if self.ui.rows.is_empty() {
    //         return;
    //     }
    //     let current = self.ui.table_state.selected().unwrap_or(0);
    //     let new_index = current.saturating_sub(amount);
    //     self.ui.table_state.select(Some(new_index));
    // }

    // fn move_selection_down(&mut self, amount: usize) {
    //     if self.ui.rows.is_empty() {
    //         return;
    //     }
    //     let current = self.ui.table_state.selected().unwrap_or(0);
    //     let max_index = self.ui.rows.len().saturating_sub(1);
    //     let new_index = (current + amount).min(max_index);
    //     self.ui.table_state.select(Some(new_index));
    // }

    // fn move_selection_to_start(&mut self) {
    //     if self.ui.rows.is_empty() {
    //         return;
    //     }
    //     self.ui.table_state.select(Some(0));
    // }

    // fn move_selection_to_end(&mut self) {
    //     if self.ui.rows.is_empty() {
    //         return;
    //     }
    //     self.ui
    //         .table_state
    //         .select(Some(self.ui.rows.len().saturating_sub(1)));
    // }

    // fn page_step(&self) -> usize {
    //     usize::from(self.ui.last_table_height.max(1))
    // }

    // fn toggle_current_row(&mut self) {
    //     let Some(selected_idx) = self.ui.table_state.selected() else {
    //         return;
    //     };
    //     let Some(row) = self.ui.rows.get(selected_idx) else {
    //         return;
    //     };

    //     if !self
    //         .selected_role_assignments
    //         .insert(row.role_assignment_id.clone())
    //     {
    //         self.selected_role_assignments
    //             .remove(&row.role_assignment_id);
    //     }
    //     self.update_status_message();
    // }

    // fn invert_selection(&mut self) {
    //     let mut new_selection = FxHashSet::default();
    //     for row in &self.ui.rows {
    //         if !self
    //             .selected_role_assignments
    //             .contains(&row.role_assignment_id)
    //         {
    //             new_selection.insert(row.role_assignment_id.clone());
    //         }
    //     }
    //     self.selected_role_assignments = new_selection;
    //     self.update_status_message();
    // }

    // fn clear_selection(&mut self) {
    //     self.selected_role_assignments.clear();
    //     self.update_status_message();
    // }

    // fn build_success_result(&mut self) -> Result<Option<RoleAssignmentPickerAppResult>> {
    //     let chosen_ids: Vec<RoleAssignmentId> = self
    //         .ui
    //         .rows
    //         .iter()
    //         .filter(|row| {
    //             self.selected_role_assignments
    //                 .contains(&row.role_assignment_id)
    //         })
    //         .map(|row| row.role_assignment_id.clone())
    //         .collect();
    //     if chosen_ids.is_empty() {
    //         return Ok(None);
    //     }

    //     if self
    //         .data
    //         .role_definitions_and_assignments
    //         .as_loaded()
    //         .is_none()
    //     {
    //         return Ok(None);
    //     }
    //     if self.data.principals.as_loaded().is_none() {
    //         return Ok(None);
    //     }

    //     let role_holder = take_loadable_value(&mut self.data.role_definitions_and_assignments)
    //         .expect("checked for loaded data above");
    //     let principal_holder =
    //         take_loadable_value(&mut self.data.principals).expect("checked for loaded data above");

    //     Ok(Some(RoleAssignmentPickerAppResult::Some {
    //         chosen_role_assignment_ids: chosen_ids,
    //         role_definitions_and_assignments: role_holder.into_inner(),
    //         principals: principal_holder.into_inner(),
    //     }))
    // }
}

fn take_loadable_value<T>(loadable: &mut Loadable<T>) -> Option<T> {
    match mem::take(loadable) {
        Loadable::Loaded { value, .. } => Some(value),
        other => {
            *loadable = other;
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::RoleAssignmentPickerApp;
    use super::RoleAssignmentPickerAppResult;

    #[tokio::test]
    #[ignore = "manual entrypoint"]
    async fn manual_pick() -> eyre::Result<()> {
        color_eyre::install()?;
        let app = RoleAssignmentPickerApp::new();
        let _result: RoleAssignmentPickerAppResult = app.run().await?;
        Ok(())
    }
}
