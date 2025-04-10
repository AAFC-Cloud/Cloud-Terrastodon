pub mod app;
pub mod app_message;
pub mod draw_app;
pub mod loadable;
pub mod state_mutator;
pub mod work;
pub mod loadable_work;

use app::MyApp;
use eframe::NativeOptions;
use eframe::egui::Vec2;
use eyre::bail;
use tokio::task::block_in_place;
use tracing::info;

pub async fn egui_main() -> eyre::Result<()> {
    info!("Hello from egui!");

    let mut native_options = NativeOptions::default();
    native_options.viewport.inner_size = Some(Vec2::new(800., 700.));
    if let Err(e) = block_in_place(move || {
        eframe::run_native(
            "MyApp",
            native_options,
            Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
        )
    }) {
        bail!("Failed to run app: {e:#?}");
    };
    Ok(())
}
