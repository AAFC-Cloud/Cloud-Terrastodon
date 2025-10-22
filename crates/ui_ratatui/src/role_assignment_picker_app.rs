use cloud_terrastodon_azure::prelude::Principal;
use cloud_terrastodon_azure::prelude::PrincipalCollection;
use cloud_terrastodon_azure::prelude::RoleAssignment;
use cloud_terrastodon_azure::prelude::RoleAssignmentId;
use cloud_terrastodon_azure::prelude::RoleDefinition;
use cloud_terrastodon_azure::prelude::RoleDefinitionsAndAssignments;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_command::app_work::AppWorkState;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eyre::Result;
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
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::time::Duration;

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
pub struct RoleAssignmentPickerApp {
    data: AppData,
    work: AppWorkState<AppData>,
    ui: UiState,
    selected_role_assignments: FxHashSet<RoleAssignmentId>,
}

impl RoleAssignmentPickerApp {
    /// Creates a new app instance.
    pub fn new() -> Self {
        Self {
            data: AppData::default(),
            work: AppWorkState::new(),
            ui: UiState::default(),
            selected_role_assignments: FxHashSet::default(),
        }
    }

    /// Runs the interactive picker. Returns the user's selection or cancellation.
    pub async fn run(mut self) -> Result<RoleAssignmentPickerAppResult> {
        self.enqueue_initial_work()?;

        let mut terminal = ratatui::init();
        terminal.clear()?;
        self.ui.status_message = "Loading role assignments and principals...".to_string();

        let result = 'outer: loop {
            // 1. Handle background work completion
            self.work.handle_messages(&mut self.data)?;

            // 2. Handle keyboard input
            while event::poll(Duration::from_millis(0))? {
                let evt = event::read()?;
                if let Some(result) = self.handle_event(evt)? {
                    break 'outer result;
                }
            }

            // 3. Rebuild rows if data changed
            if self.data.changed {
                self.rebuild_rows_if_ready();
            }

            // 4. Draw the UI
            terminal.draw(|frame| self.draw(frame))?;

            // 5. Throttle loop to avoid busy waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        };

        ratatui::restore();
        self.work.work_tracker.finish().await?;
        Ok(result)
    }

    fn enqueue_initial_work(&mut self) -> Result<()> {
        let role_work = LoadableWorkBuilder::<AppData, RoleDefinitionsHolder>::new()
            .description("fetch_all_role_definitions_and_assignments")
            .setter(|state, value| {
                state.role_definitions_and_assignments = value;
                state.changed = true;
            })
            .work(async move {
                let data = fetch_all_role_definitions_and_assignments().await?;
                Ok(RoleDefinitionsHolder(data))
            })
            .build()?;
        role_work.enqueue(&self.work, &mut self.data)?;

        let principals_work = LoadableWorkBuilder::<AppData, PrincipalCollectionHolder>::new()
            .description("fetch_all_principals")
            .setter(|state, value| {
                state.principals = value;
                state.changed = true;
            })
            .work(async move {
                let data = fetch_all_principals().await?;
                Ok(PrincipalCollectionHolder(data))
            })
            .build()?;
        principals_work.enqueue(&self.work, &mut self.data)?;

        Ok(())
    }

    fn rebuild_rows_if_ready(&mut self) {
        let Some(role_data) = self.data.role_definitions_and_assignments.as_loaded() else {
            self.ui.rows.clear();
            self.ui.status_message = "Loading role assignments...".to_string();
            return;
        };
        let Some(principals) = self.data.principals.as_loaded() else {
            self.ui.rows.clear();
            self.ui.status_message = "Loading principals...".to_string();
            return;
        };

        let mut rows: Vec<RoleAssignmentRow> = role_data
            .iter_role_assignments()
            .map(|(assignment, definition)| {
                RoleAssignmentRow::from_assignment(assignment, definition, principals.deref())
            })
            .collect();

        rows.sort_by(|left, right| {
            left.scope
                .cmp(&right.scope)
                .then_with(|| left.principal_name.cmp(&right.principal_name))
                .then_with(|| left.role_display_name.cmp(&right.role_display_name))
        });

        let valid_ids: FxHashSet<RoleAssignmentId> = rows
            .iter()
            .map(|row| row.role_assignment_id.clone())
            .collect();
        self.selected_role_assignments
            .retain(|id| valid_ids.contains(id));

        self.ui.rows = rows;
        self.ensure_selection_is_within_bounds();
        self.update_status_message();
        self.data.changed = false;
    }

    fn ensure_selection_is_within_bounds(&mut self) {
        if self.ui.rows.is_empty() {
            self.ui.table_state.select(None);
            return;
        }

        match self.ui.table_state.selected() {
            None => self.ui.table_state.select(Some(0)),
            Some(idx) if idx >= self.ui.rows.len() => {
                self.ui.table_state.select(Some(self.ui.rows.len() - 1));
            }
            _ => {}
        }
    }

    fn update_status_message(&mut self) {
        if self.ui.rows.is_empty() {
            self.ui.status_message = "No role assignments found.".to_string();
            return;
        }

        let selected = self.selected_role_assignments.len();
        let total = self.ui.rows.len();
        self.ui.status_message = format!(
            "Loaded {total} role assignments | Selected {selected} | Esc to cancel, Enter to accept"
        );
    }

    fn draw(&mut self, frame: &mut Frame) {
        let region = frame.area();

        let status_paragraph = Paragraph::new(self.ui.status_message.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Role Assignment Picker"),
        );

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(region);
        let status_area = chunks[0];
        let table_area = chunks[1];

        frame.render_widget(status_paragraph, status_area);

        self.ui.last_table_height = table_area.height.saturating_sub(2).max(1);

        let table = {
            let header = Row::new(vec![
                Cell::from("Sel"),
                Cell::from("Principal"),
                Cell::from("Principal Type"),
                Cell::from("Role"),
                Cell::from("Scope"),
            ])
            .style(Style::default().add_modifier(Modifier::BOLD));

            let rows: Vec<Row<'_>> = self
                .ui
                .rows
                .iter()
                .map(|row| {
                    let is_marked = self
                        .selected_role_assignments
                        .contains(&row.role_assignment_id);
                    RoleAssignmentRow::to_table_row(row, is_marked)
                })
                .collect();

            Table::new(
                rows,
                [
                    Constraint::Length(4),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Length(14),
                    Constraint::Percentage(25),
                ],
            )
            .header(header)
            .block(Block::default().borders(Borders::ALL))
            .highlight_symbol("▶ ")
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        };
        frame.render_stateful_widget(table, table_area, &mut self.ui.table_state);
    }

    fn handle_event(&mut self, event: Event) -> Result<Option<RoleAssignmentPickerAppResult>> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if key.code == KeyCode::Char('c') {
                        return Ok(Some(RoleAssignmentPickerAppResult::Cancelled));
                    }
                    if key.code == KeyCode::Char('a') {
                        self.select_all_rows();
                        return Ok(None);
                    }
                    if key.code == KeyCode::Char('d') {
                        self.clear_selection();
                        return Ok(None);
                    }
                    if key.code == KeyCode::Char('t') {
                        self.invert_selection();
                        return Ok(None);
                    }
                }

                match key.code {
                    KeyCode::Esc => return Ok(Some(RoleAssignmentPickerAppResult::Cancelled)),
                    KeyCode::Up => self.move_selection_up(1),
                    KeyCode::Down => self.move_selection_down(1),
                    KeyCode::PageUp => self.move_selection_up(self.page_step()),
                    KeyCode::PageDown => self.move_selection_down(self.page_step()),
                    KeyCode::Home => self.move_selection_to_start(),
                    KeyCode::End => self.move_selection_to_end(),
                    KeyCode::Tab => {
                        self.toggle_current_row();
                        self.move_selection_down(1);
                    }
                    KeyCode::Enter => {
                        if let Some(result) = self.build_success_result()? {
                            return Ok(Some(result));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn move_selection_up(&mut self, amount: usize) {
        if self.ui.rows.is_empty() {
            return;
        }
        let current = self.ui.table_state.selected().unwrap_or(0);
        let new_index = current.saturating_sub(amount);
        self.ui.table_state.select(Some(new_index));
    }

    fn move_selection_down(&mut self, amount: usize) {
        if self.ui.rows.is_empty() {
            return;
        }
        let current = self.ui.table_state.selected().unwrap_or(0);
        let max_index = self.ui.rows.len().saturating_sub(1);
        let new_index = (current + amount).min(max_index);
        self.ui.table_state.select(Some(new_index));
    }

    fn move_selection_to_start(&mut self) {
        if self.ui.rows.is_empty() {
            return;
        }
        self.ui.table_state.select(Some(0));
    }

    fn move_selection_to_end(&mut self) {
        if self.ui.rows.is_empty() {
            return;
        }
        self.ui
            .table_state
            .select(Some(self.ui.rows.len().saturating_sub(1)));
    }

    fn page_step(&self) -> usize {
        usize::from(self.ui.last_table_height.max(1))
    }

    fn toggle_current_row(&mut self) {
        let Some(selected_idx) = self.ui.table_state.selected() else {
            return;
        };
        let Some(row) = self.ui.rows.get(selected_idx) else {
            return;
        };

        if !self
            .selected_role_assignments
            .insert(row.role_assignment_id.clone())
        {
            self.selected_role_assignments
                .remove(&row.role_assignment_id);
        }
        self.update_status_message();
    }

    fn select_all_rows(&mut self) {
        self.selected_role_assignments = self
            .ui
            .rows
            .iter()
            .map(|row| row.role_assignment_id.clone())
            .collect();
        self.update_status_message();
    }

    fn invert_selection(&mut self) {
        let mut new_selection = FxHashSet::default();
        for row in &self.ui.rows {
            if !self
                .selected_role_assignments
                .contains(&row.role_assignment_id)
            {
                new_selection.insert(row.role_assignment_id.clone());
            }
        }
        self.selected_role_assignments = new_selection;
        self.update_status_message();
    }

    fn clear_selection(&mut self) {
        self.selected_role_assignments.clear();
        self.update_status_message();
    }

    fn build_success_result(&mut self) -> Result<Option<RoleAssignmentPickerAppResult>> {
        let chosen_ids: Vec<RoleAssignmentId> = self
            .ui
            .rows
            .iter()
            .filter(|row| {
                self.selected_role_assignments
                    .contains(&row.role_assignment_id)
            })
            .map(|row| row.role_assignment_id.clone())
            .collect();
        if chosen_ids.is_empty() {
            return Ok(None);
        }

        if self
            .data
            .role_definitions_and_assignments
            .as_loaded()
            .is_none()
        {
            return Ok(None);
        }
        if self.data.principals.as_loaded().is_none() {
            return Ok(None);
        }

        let role_holder = take_loadable_value(&mut self.data.role_definitions_and_assignments)
            .expect("checked for loaded data above");
        let principal_holder =
            take_loadable_value(&mut self.data.principals).expect("checked for loaded data above");

        Ok(Some(RoleAssignmentPickerAppResult::Some {
            chosen_role_assignment_ids: chosen_ids,
            role_definitions_and_assignments: role_holder.into_inner(),
            principals: principal_holder.into_inner(),
        }))
    }
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

#[derive(Default)]
struct AppData {
    role_definitions_and_assignments: Loadable<RoleDefinitionsHolder>,
    principals: Loadable<PrincipalCollectionHolder>,
    changed: bool,
}

#[derive(Default)]
struct UiState {
    table_state: TableState,
    rows: Vec<RoleAssignmentRow>,
    status_message: String,
    last_table_height: u16,
}

#[derive(Clone)]
struct RoleAssignmentRow {
    role_assignment_id: RoleAssignmentId,
    scope: String,
    principal_name: String,
    principal_kind: String,
    role_display_name: String,
}

impl RoleAssignmentRow {
    fn from_assignment(
        assignment: &RoleAssignment,
        definition: &RoleDefinition,
        principals: &PrincipalCollection,
    ) -> Self {
        let principal_info = principals
            .get(&assignment.principal_id)
            .map(principal_details)
            .unwrap_or_else(|| ("Unknown".to_string(), "Unknown".to_string()));

        Self {
            role_assignment_id: assignment.id.clone(),
            scope: assignment.scope.short_form(),
            principal_name: principal_info.0,
            principal_kind: principal_info.1,
            role_display_name: definition.display_name.clone(),
        }
    }

    fn to_table_row<'a>(row: &'a Self, is_marked: bool) -> Row<'a> {
        let marker = if is_marked { "[x]" } else { "[ ]" };

        let mut table_row = Row::new(vec![
            Cell::from(marker.to_string()),
            Cell::from(row.principal_name.as_str()),
            Cell::from(row.principal_kind.as_str()),
            Cell::from(row.role_display_name.as_str()),
            Cell::from(row.scope.as_str()),
        ]);

        if is_marked {
            table_row = table_row.style(Style::default().fg(Color::Green));
        }
        table_row
    }
}

fn principal_details(principal: &Principal) -> (String, String) {
    match principal {
        Principal::User(user) => (user.user_principal_name.clone(), "User".to_string()),
        Principal::Group(group) => (group.display_name.clone(), "Group".to_string()),
        Principal::ServicePrincipal(sp) => {
            (sp.display_name.clone(), "Service Principal".to_string())
        }
    }
}

struct RoleDefinitionsHolder(RoleDefinitionsAndAssignments);

impl RoleDefinitionsHolder {
    fn into_inner(self) -> RoleDefinitionsAndAssignments {
        self.0
    }
}

impl Deref for RoleDefinitionsHolder {
    type Target = RoleDefinitionsAndAssignments;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for RoleDefinitionsHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoleDefinitionsHolder").finish()
    }
}

struct PrincipalCollectionHolder(PrincipalCollection);

impl PrincipalCollectionHolder {
    fn into_inner(self) -> PrincipalCollection {
        self.0
    }
}

impl Deref for PrincipalCollectionHolder {
    type Target = PrincipalCollection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for PrincipalCollectionHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrincipalCollectionHolder").finish()
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
