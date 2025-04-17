pub mod app;
pub mod app_message;
pub mod autosave_info;
pub mod draw_app;
pub mod file_drag_and_drop;
pub mod icons;
pub mod loadable;
pub mod loadable_work;
pub mod state_mutator;
pub mod widgets;
pub mod windows;
pub mod work;
pub mod workers;
use app::MyApp;
use cloud_terrastodon_core_pathing::AppDir;
use eframe::NativeOptions;
use eyre::bail;
use tokio::runtime;
use tokio::task::block_in_place;
use tracing::info;

pub async fn egui_main() -> eyre::Result<()> {
    info!("Hello from egui!");

    let mut native_options = NativeOptions::default();
    native_options.persist_window = true;
    native_options.persistence_path = Some(AppDir::Config.join("egui_window_state.ron"));

    if let Err(e) = block_in_place(move || {
        eframe::run_native(
            "MyApp",
            native_options,
            Box::new(|cc: &eframe::CreationContext<'_>| {
                // This gives us image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);
                let app =
                    runtime::Handle::current().block_on(async move { MyApp::new(cc).await })?;
                Ok(Box::new(app))
            }),
        )
    }) {
        bail!("Failed to run app: {e:#?}");
    };
    Ok(())
}
