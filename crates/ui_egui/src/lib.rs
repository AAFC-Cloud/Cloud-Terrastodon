pub mod app;
pub mod app_message;
pub mod autosave_info;
pub mod draw_app;
pub mod file_drag_and_drop;
pub mod icons;
pub mod loadable;
pub mod loadable_work;
pub mod run_app;
pub mod state_mutator;
pub mod widgets;
pub mod windows;
pub mod work;
pub mod work_tracker;
pub mod workers;

use tracing::info;
pub async fn egui_main() -> eyre::Result<()> {
    info!("Hello from egui!");
    run_app::run_app().await?;
    Ok(())
}
