use cloud_terrastodon_command::app_work::AppWorkState;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::{self};
use ratatui::prelude::*;
use std::any::type_name_of_val;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Default)]
pub struct MyApp {
    pub data: MyAppData,
    pub work: AppWorkState<MyAppData>,
}

#[derive(Default)]
pub struct MyAppData {
    pub thingies: Loadable<Vec<u16>>,
}

pub async fn fetch_all_thingies() -> eyre::Result<Vec<u16>> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok(vec![1, 2, 3, 4, 5])
}

impl MyApp {
    pub async fn new() -> eyre::Result<Self> {
        Ok(Self::default())
    }

    pub async fn run(mut self) -> eyre::Result<()> {
        LoadableWorkBuilder::<MyAppData, Vec<u16>>::new()
            .description(type_name_of_val(&fetch_all_thingies))
            .setter(|state, value| state.thingies = value)
            .work(async {
                let thingies = fetch_all_thingies().await?;
                Ok(thingies)
            })
            .build()?
            .enqueue(&self.work, &mut self.data)?;

        let mut terminal = ratatui::init();
        terminal.clear()?;

        'outer: loop {
            // Handle any messages from background work
            self.work.handle_messages(&mut self.data)?;

            // Handle any user input
            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()?
                    && key.kind == KeyEventKind::Press
                {
                    match key.code {
                        KeyCode::Esc => {
                            break 'outer;
                        }
                        _ => {}
                    }
                }
            }

            // Draw to the terminal
            let thingy_display = match &self.data.thingies {
                Loadable::NotLoaded => "Not loaded".to_string(),
                Loadable::Loading { started_at } => format!(
                    "Loading{}",
                    ".".repeat(((Instant::now() - *started_at).as_secs_f32() * 2.0) as usize % 4)
                ),
                Loadable::Loaded {
                    value,
                    started_at,
                    finished_at,
                } => format!(
                    "{} thingies loaded in {:?}",
                    value.len(),
                    *finished_at - *started_at
                ),
                Loadable::Failed { error, .. } => format!("Error: {error}"),
            };
            let work_display = format!("{} work items in progress", self.work.work_tracker.len()?);
            terminal.draw(|f| {
                Text::from(format!(
                    "Thingies: {thingy_display}\nPress Esc to exit\n{work_display}",
                ))
                .render(f.area(), f.buffer_mut());
            })?;

            // Sleep between iterations
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        ratatui::restore();
        debug!("Pre-finish");
        self.work.work_tracker.finish().await?;
        debug!("Post-finish");
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .with_file(true)
        .with_target(false)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .without_time()
        .init();
    let app = MyApp::new().await?;
    app.run().await?;
    Ok(())
}