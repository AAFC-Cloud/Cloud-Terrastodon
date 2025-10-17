use crossterm::event::EventStream;
use eyre::Result;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::prelude::Constraint;
use ratatui::prelude::Layout;
use ratatui::prelude::Line;
use ratatui::prelude::Stylize;
use std::time::Duration;
use tracing::info;

pub async fn ui_main() -> Result<()> {
    info!("Hi there!");
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal).await;
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
struct App {
    should_quit: bool,
}

impl App {
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

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [title_area, _body_area] = vertical.areas(frame.area());
        let title = Line::from("Ratatui async example").centered().bold();
        frame.render_widget(title, title_area);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('h') => info!("hi!"),
                KeyCode::Char('r') => ratatui::restore(),
                _ => {}
            }
        }
    }
}
