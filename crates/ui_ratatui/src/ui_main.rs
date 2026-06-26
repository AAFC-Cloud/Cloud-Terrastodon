use cloud_terrastodon_registry::InputDependency;
use cloud_terrastodon_registry::KNOWN_THINGS;
use cloud_terrastodon_registry::Thing;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::things_producing;
use crossterm::event::EventStream;
use eyre::Result;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Alignment;
use ratatui::prelude::Constraint;
use ratatui::prelude::Layout;
use ratatui::prelude::Modifier;
use ratatui::prelude::Rect;
use ratatui::prelude::Style;
use ratatui::prelude::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use std::time::Duration;
use tracing::info;

pub async fn ui_main() -> Result<()> {
    info!("Starting object browser");
    let terminal = ratatui::init();
    let app_result = ObjectBrowserApp::default().run(terminal).await;
    ratatui::restore();
    app_result
}

struct ObjectBrowserApp {
    should_quit: bool,
    mode: UiMode,
    command_palette_state: ListState,
    create_object_state: ListState,
    arena_objects: Vec<ArenaObject>,
    next_object_id: usize,
    status_message: String,
}

impl Default for ObjectBrowserApp {
    fn default() -> Self {
        let mut command_palette_state = ListState::default();
        command_palette_state.select(Some(0));
        let mut create_object_state = ListState::default();
        create_object_state.select(Some(0));
        Self {
            should_quit: false,
            mode: UiMode::Arena,
            command_palette_state,
            create_object_state,
            arena_objects: Vec::new(),
            next_object_id: 1,
            status_message: "Press Space to open the command palette.".to_string(),
        }
    }
}

impl ObjectBrowserApp {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [title_area, body_area, status_area] = vertical.areas(frame.area());

        let title = Line::from("Cloud Terrastodon Object Browser")
            .centered()
            .bold();
        frame.render_widget(title, title_area);

        self.draw_arena(frame, body_area);

        let status = Line::from(self.status_message.as_str());
        frame.render_widget(status, status_area);

        match self.mode {
            UiMode::Arena => {}
            UiMode::CommandPalette => self.draw_command_palette(frame),
            UiMode::CreateObject => self.draw_create_object(frame),
            UiMode::InvokeThing(thing) => self.draw_invoke_thing(frame, thing),
            UiMode::ConstructThing(thing) => self.draw_construct_thing(frame, thing),
        }
    }

    fn draw_arena(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Object Arena");
        if self.arena_objects.is_empty() {
            let text = Text::from(vec![
                Line::from("The object arena is empty."),
                Line::from("Press Space to open the command palette."),
            ]);
            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        let items = self
            .arena_objects
            .iter()
            .map(|object| {
                ListItem::new(format!(
                    "#{} {} ({})",
                    object.id, object.label, object.shape_name
                ))
            })
            .collect::<Vec<_>>();
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn draw_command_palette(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(Clear, area);
        let items = CommandAction::all()
            .iter()
            .map(|action| ListItem::new(action.label()))
            .collect::<Vec<_>>();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Palette"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut self.command_palette_state);
    }

    fn draw_create_object(&mut self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);
        let items = registry_entries()
            .into_iter()
            .map(|thing| ListItem::new(thing_label(thing)))
            .collect::<Vec<_>>();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Create New Object"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut self.create_object_state);
    }

    fn draw_invoke_thing(&self, frame: &mut Frame, thing: &'static Thing) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);
        let Some(invocation) = thing.invocation else {
            return;
        };
        let mut lines = vec![
            Line::from(Span::styled(
                describe_shape(thing.shape),
                Style::default().bold(),
            )),
            Line::from(format!(
                "Output: {}",
                describe_shape(invocation.output_shape)
            )),
            Line::from(""),
            Line::from("Required fields"),
        ];

        let dependencies = thing.input_dependencies();
        if dependencies.is_empty() {
            lines.push(Line::from("No reflected input fields."));
        } else {
            for dependency in dependencies {
                lines.push(argument_line(dependency));
                let producers = things_producing(dependency.shape);
                for producer in producers {
                    lines.push(Line::from(format!(
                        "  producer: {}",
                        describe_shape(producer.shape)
                    )));
                }
            }
        }

        let paragraph = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Invocation Parameters"),
        );
        frame.render_widget(paragraph, area);
    }

    fn draw_construct_thing(&self, frame: &mut Frame, thing: &'static Thing) {
        let area = centered_rect(70, 45, frame.area());
        frame.render_widget(Clear, area);
        let paragraph = Paragraph::new(vec![
            Line::from(Span::styled(
                describe_shape(thing.shape),
                Style::default().bold(),
            )),
            Line::from(""),
            Line::from("This is where reflected field-by-field construction will happen."),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Construct Object"),
        );
        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: &Event) {
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match self.mode {
            UiMode::Arena => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char(' ') => self.open_command_palette(),
                _ => {}
            },
            UiMode::CommandPalette => match key.code {
                KeyCode::Esc => self.close_overlay(),
                KeyCode::Up => self.command_palette_state.select_previous(),
                KeyCode::Down => self.command_palette_state.select_next(),
                KeyCode::Enter => self.activate_command_palette_selection(),
                _ => {}
            },
            UiMode::CreateObject => match key.code {
                KeyCode::Esc => self.open_command_palette(),
                KeyCode::Up => self.create_object_state.select_previous(),
                KeyCode::Down => self.create_object_state.select_next(),
                KeyCode::Enter => self.activate_create_object_selection(),
                _ => {}
            },
            UiMode::InvokeThing(_) | UiMode::ConstructThing(_) => match key.code {
                KeyCode::Esc => self.mode = UiMode::CreateObject,
                KeyCode::Enter => self.record_placeholder_object(),
                _ => {}
            },
        }
    }

    fn open_command_palette(&mut self) {
        self.command_palette_state.select(Some(0));
        self.mode = UiMode::CommandPalette;
        self.status_message = "Command palette".to_string();
    }

    fn close_overlay(&mut self) {
        self.mode = UiMode::Arena;
        self.status_message = "Press Space to open the command palette.".to_string();
    }

    fn activate_command_palette_selection(&mut self) {
        match selected_index(&self.command_palette_state, CommandAction::all().len())
            .and_then(CommandAction::get)
        {
            Some(CommandAction::CreateObject) => {
                self.create_object_state.select(Some(0));
                self.mode = UiMode::CreateObject;
                self.status_message = "Create new object".to_string();
            }
            None => {}
        }
    }

    fn activate_create_object_selection(&mut self) {
        let entries = registry_entries();
        let Some(thing) = selected_index(&self.create_object_state, entries.len())
            .and_then(|index| entries.get(index).copied())
        else {
            return;
        };

        if thing.is_invokable() {
            self.mode = UiMode::InvokeThing(thing);
            self.status_message = format!("Picking arguments for {}", describe_shape(thing.shape));
        } else {
            self.mode = UiMode::ConstructThing(thing);
            self.status_message = format!("Constructing {}", describe_shape(thing.shape));
        }
    }

    fn record_placeholder_object(&mut self) {
        let (label, shape_name) = match self.mode {
            UiMode::InvokeThing(thing) => (
                format!("Invocation: {}", describe_shape(thing.shape)),
                describe_shape(thing.shape),
            ),
            UiMode::ConstructThing(thing) => (
                format!("Object: {}", describe_shape(thing.shape)),
                describe_shape(thing.shape),
            ),
            _ => return,
        };

        self.arena_objects.push(ArenaObject {
            id: self.next_object_id,
            label,
            shape_name,
        });
        self.next_object_id += 1;
        self.mode = UiMode::Arena;
        self.status_message = "Added a placeholder arena object.".to_string();
    }
}

#[derive(Clone, Copy)]
enum UiMode {
    Arena,
    CommandPalette,
    CreateObject,
    InvokeThing(&'static Thing),
    ConstructThing(&'static Thing),
}

#[derive(Clone, Copy)]
enum CommandAction {
    CreateObject,
}

impl CommandAction {
    fn all() -> &'static [Self] {
        &[Self::CreateObject]
    }

    fn get(index: usize) -> Option<Self> {
        Self::all().get(index).copied()
    }

    fn label(self) -> &'static str {
        match self {
            Self::CreateObject => "Create new object",
        }
    }
}

struct ArenaObject {
    id: usize,
    label: String,
    shape_name: String,
}

fn registry_entries() -> Vec<&'static Thing> {
    KNOWN_THINGS.iter().collect()
}

fn thing_label(thing: &Thing) -> String {
    if let Some(output_shape) = thing.output_shape() {
        format!(
            "thing   {} => {}",
            describe_shape(thing.shape),
            describe_shape(output_shape)
        )
    } else {
        format!("thing   {}", describe_shape(thing.shape))
    }
}

fn argument_line(dependency: InputDependency) -> Line<'static> {
    Line::from(format!(
        "{}: {}",
        dependency.field_name,
        describe_shape(dependency.shape)
    ))
}

fn selected_index(state: &ListState, len: usize) -> Option<usize> {
    state.selected().filter(|index| *index < len)
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ]);
    let [_, center, _] = vertical.areas(area);
    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]);
    let [_, middle, _] = horizontal.areas(center);
    middle
}

#[cfg(test)]
mod tests {
    use super::registry_entries;
    use cloud_terrastodon_registry::KNOWN_THINGS;

    #[test]
    fn registry_entries_include_things() {
        let entries = registry_entries();
        assert_eq!(entries.len(), KNOWN_THINGS.len());
    }
}
