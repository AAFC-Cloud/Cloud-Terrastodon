pub mod app;
pub mod app_message;
pub mod draw_app;
pub mod icons;
pub mod loadable;
pub mod loadable_work;
pub mod state_mutator;
pub mod widgets;
pub mod windows;
pub mod work;
pub mod workers;
pub mod autosave_info;

use app::MyApp;
use cloud_terrastodon_core_config::egui_config::EguiConfig;
use cloud_terrastodon_core_config::iconfig::IConfig;
use cloud_terrastodon_core_pathing::AppDir;
use eframe::NativeOptions;
use eyre::bail;
use tokio::task::block_in_place;
use tracing::info;

pub async fn egui_main() -> eyre::Result<()> {
    info!("Hello from egui!");

    let config = EguiConfig::load().await?;

    let mut native_options = NativeOptions::default();
    native_options.persist_window = true;
    native_options.persistence_path = Some(AppDir::Config.join("egui_window_state.ron"));

    if let Err(e) = block_in_place(move || {
        eframe::run_native(
            "MyApp",
            native_options,
            Box::new(|cc| {
                // This gives us image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);

                Ok(Box::new(MyApp::new(cc, config)))
            }),
        )
    }) {
        bail!("Failed to run app: {e:#?}");
    };
    Ok(())
}
