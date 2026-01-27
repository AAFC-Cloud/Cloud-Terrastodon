use crate::app::MyApp;
use crate::work_tracker::WorkTracker;
use cloud_terrastodon_pathing::AppDir;
use eframe::NativeOptions;
use eyre::bail;
use eyre::eyre;
use std::rc::Rc;
use tokio::runtime;
use tokio::task::block_in_place;
use tracing::info;

pub async fn run_app(app_info: String) -> eyre::Result<()> {
    let native_options = NativeOptions {
        persist_window: true,
        persistence_path: Some(AppDir::Config.join("egui_window_state.ron")),
        run_and_return: true,
        ..Default::default()
    };

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
                        .block_on(async move { MyApp::new(cc, work_tracker2, app_info).await })?;
                    Ok(Box::new(app))
                }),
            )
        }) {
            let work_tracker =
                Rc::try_unwrap(work_tracker).map_err(|_| eyre!("Failed to take work_tracker"))?;
            work_tracker.finish().await?;
            bail!("Failed to run app: {e:#?}");
        } else {
            let work_tracker =
                Rc::try_unwrap(work_tracker).map_err(|_| eyre!("Failed to take work_tracker"))?;
            work_tracker.finish().await?;
        };
    }

    info!("Goodbye!");

    Ok(())
}
