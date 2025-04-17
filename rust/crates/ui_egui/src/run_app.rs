use crate::app::MyApp;
use crate::work_tracker::WorkTracker;
use cloud_terrastodon_core_pathing::AppDir;
use eframe::NativeOptions;
use eyre::bail;
use std::rc::Rc;
use tokio::runtime;
use tokio::task::block_in_place;
use tracing::info;

pub async fn run_app() -> eyre::Result<()> {
    let mut native_options = NativeOptions::default();
    native_options.persist_window = true;
    native_options.persistence_path = Some(AppDir::Config.join("egui_window_state.ron"));
    native_options.run_and_return = true;

    let work_tracker = Rc::new(WorkTracker::new());

    {
        let work_tracker2 = work_tracker.clone();
        if let Err(e) = block_in_place(move || {
            eframe::run_native(
                "MyApp",
                native_options,
                Box::new(|cc: &eframe::CreationContext<'_>| {
                    // This gives us image support:
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    let app = runtime::Handle::current()
                        .block_on(async move { MyApp::new(cc, work_tracker2).await })?;
                    Ok(Box::new(app))
                }),
            )
        }) {
            work_tracker.finish().await?;
            bail!("Failed to run app: {e:#?}");
        } else {
            work_tracker.finish().await?;
        };
    }

    info!("Goodbye!");

    Ok(())
}
