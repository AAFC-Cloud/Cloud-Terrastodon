use crate::app::MyApp;
use crate::work_tracker::WorkTracker;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_pathing::AppDir;
use eframe::NativeOptions;
use eframe::egui::ViewportCommand;
use eframe::egui::ViewportId;
use eyre::eyre;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::panic::resume_unwind;
use std::rc::Rc;
use tokio::runtime;
use tokio::task::block_in_place;
use tracing::info;

pub async fn run_app(
    app_info: String,
    tenant_id: AzureTenantId,
    auth_context: AuthContext,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let (_shutdown, run) = executor.with_async_child_scope(move |executor| async move {
        let native_options = NativeOptions {
            persist_window: true,
            persistence_path: Some(AppDir::Config.join("egui_window_state.ron")),
            run_and_return: true,
            ..Default::default()
        };

        let work_tracker = Rc::new(WorkTracker::new());

        {
            let work_tracker2 = work_tracker.clone();
            let app_executor = executor.clone();
            let result = catch_unwind(AssertUnwindSafe(|| {
                block_in_place(move || {
                    // Keep the wakeup registered until the native event loop
                    // exits, including while no UI frame is otherwise due.
                    let mut _shutdown_listener = None;
                    eframe::run_native(
                        "MyApp",
                        native_options,
                        Box::new(|cc: &eframe::CreationContext<'_>| {
                            let gui = cc.egui_ctx.clone();
                            _shutdown_listener =
                                Some(app_executor.cancellation().register_listener(move |_| {
                                    // Sending a viewport command also requests its
                                    // repaint, waking an idle eframe event loop.
                                    gui.send_viewport_cmd_to(
                                        ViewportId::ROOT,
                                        ViewportCommand::Close,
                                    );
                                }));
                            // This gives us image support:
                            egui_extras::install_image_loaders(&cc.egui_ctx);
                            let app = runtime::Handle::current().block_on(async move {
                                MyApp::new(
                                    cc,
                                    tenant_id,
                                    auth_context,
                                    work_tracker2,
                                    app_info,
                                    &app_executor,
                                )
                                .await
                            })?;
                            Ok(Box::new(app))
                        }),
                    )
                })
            }));
            executor.cancellation().request_cancel("GUI window closed");
            let drained = executor
                .cancellation()
                .finish_required("drain GUI background work", async |_cleanup| {
                    work_tracker.finish().await
                })
                .await;
            match (result, drained) {
                (Err(panic), drain) => {
                    if let Err(drain_error) = drain {
                        tracing::error!(
                            ?drain_error,
                            "Background work failed while draining after a GUI panic"
                        );
                    }
                    resume_unwind(panic);
                }
                (Ok(Err(error)), drain) => {
                    if let Err(drain_error) = drain {
                        tracing::error!(
                            ?drain_error,
                            "Background work also failed during GUI shutdown"
                        );
                    }
                    // eframe can contain a non-Send glutin error, so its
                    // diagnostic must cross eyre's Send + Sync boundary as text.
                    return Err(eyre!("Failed to run app: {error:#?}"));
                }
                (Ok(Ok(())), drain) => drain?,
            }
        }

        info!("Goodbye!");

        Ok(())
    });
    // This sibling lease retains the GUI owner through shutdown without making
    // independently created SDK jobs inherit a blanket cancellation deferral.
    executor
        .cancellation()
        .deferred(
            "close GUI and drain background work",
            async move |_lifetime| run.await,
        )
        .await
}
