use super::*;
use crate::object_browser::excalidraw_render_session::ExcalidrawRenderSession;
use crate::object_explorer::ArenaQueryContext;
use crate::object_explorer::ExplorerEngine;
use cloud_terrastodon_command::CancellationToken;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use facet::Facet;
use std::time::Duration;

#[derive(Clone, Debug, Facet)]
struct ExportFixture {
    name: String,
    count: u32,
}

async fn wait_for_render(app: &mut ObjectBrowserApp) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            app.tick().await.unwrap();
            if !app.excalidraw.as_ref().unwrap().is_rendering() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("render completes while engine is polled");
    assert!(
        app.status.starts_with("Rendered Excalidraw:"),
        "{}",
        app.status
    );
}

#[tokio::test]
async fn hotkeys_render_slot_and_whole_tab_with_fields_and_retry_a_busy_file() -> eyre::Result<()> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("objects.excalidraw");
    let cancellation = CancellationToken::new();
    cancellation
        .with_async_scope(|scope| async move {
            let engine = ExplorerEngine::empty();
            let (context, inbox) = ArenaQueryContext::channel(32);
            let client = async move {
                let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
                app.excalidraw = Some(ExcalidrawRenderSession::new(Some(path.clone()), &scope));
                let key = |character| {
                    Event::Key(KeyEvent::new(
                        KeyCode::Char(character),
                        KeyModifiers::CONTROL,
                    ))
                };
                app.handle_event(&key('o')).await.unwrap();
                assert!(
                    !path.exists(),
                    "the new-slot placeholder does not create a drawing"
                );
                let first = app
                    .controller
                    .insert_owned(ExportFixture {
                        name: "focused object".into(),
                        count: 1,
                    })
                    .await
                    .unwrap();
                for count in 2..=53 {
                    app.controller
                        .insert_owned(ExportFixture {
                            name: format!("object number {count}"),
                            count,
                        })
                        .await
                        .unwrap();
                }
                app.controller
                    .focus(CardAddress::Value(ValueAddress::root(first)))
                    .unwrap();
                app.handle_event(&key('o')).await.unwrap();
                wait_for_render(&mut app).await;
                let focused = std::fs::read_to_string(&path).unwrap();
                assert!(focused.contains("focused object"));
                assert!(focused.contains("count: u32"));
                assert!(!focused.contains("object number 53"));
                let lock = directory.path().join("objects.excalidraw.sync-lock");
                std::fs::write(&lock, "test-owned lock").unwrap();
                app.handle_event(&key('u')).await.unwrap();
                for _ in 0..5 {
                    app.tick().await.unwrap();
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                assert_eq!(std::fs::read_to_string(&path).unwrap(), focused);
                assert!(app.excalidraw.as_ref().unwrap().is_rendering());
                std::fs::remove_file(&lock).unwrap();
                wait_for_render(&mut app).await;
                let all = std::fs::read_to_string(&path).unwrap();
                assert!(
                    all.contains("object number 53"),
                    "tab export includes off-screen objects"
                );
                assert!(all.contains("focused object"));
                assert!(!all.contains("Target search"));
                app.close().await.unwrap();
            };
            let (_, ()) = tokio::join!(
                async {
                    engine
                        .run(inbox)
                        .await
                        .expect("synthetic engine should finish")
                },
                client
            );
            Ok(())
        })
        .await
}
