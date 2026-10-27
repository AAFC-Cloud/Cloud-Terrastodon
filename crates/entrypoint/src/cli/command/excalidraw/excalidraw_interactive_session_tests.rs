use super::*;
use cloud_terrastodon_command::CancellationToken;

#[tokio::test]
async fn external_answers_and_pending_status_survive_publication_without_feedback() -> Result<()> {
    let directory = tempfile::tempdir()?;
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let mut session = ExcalidrawInteractiveSession::open(
                directory.path().join("interactive.excalidraw"),
                &scope,
            )
            .await?;
            assert_eq!(
                session.questions.get("#UserQuestion1").map(String::as_str),
                Some("waffles")
            );
            session
                .file
                .update(|current| Ok(Some(current.unwrap().replace("waffles", "pancakes"))))?;
            session.change_test_status();
            session.poll(&scope).await;
            assert_eq!(
                session.questions.get("#UserQuestion1").map(String::as_str),
                Some("pancakes")
            );
            assert!(session.pending_test_status.is_none());
            let published = session.file.read()?.unwrap();
            assert!(published.contains("Space pressed 1"));
            assert!(published.contains("pancakes"));
            session.poll(&scope).await;
            session.poll(&scope).await;
            assert_eq!(
                session.file.read()?.unwrap(),
                published,
                "reading our own update must settle without another write"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn invalid_or_missing_external_file_preserves_pending_edits_until_restored() -> Result<()> {
    let directory = tempfile::tempdir()?;
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let mut session = ExcalidrawInteractiveSession::open(
                directory.path().join("interactive.excalidraw"),
                &scope,
            )
            .await?;
            let original = session.snapshot.clone();
            session.change_test_status();
            // An editor that does not use the shared protocol can temporarily
            // leave malformed JSON. CT must wait rather than replace that data.
            std::fs::write(session.file.path(), "{unfinished")?;
            session.poll(&scope).await;
            assert!(session.pending_test_status.is_some());
            assert_eq!(session.snapshot, original);
            assert_eq!(std::fs::read_to_string(session.file.path())?, "{unfinished");
            std::fs::remove_file(session.file.path())?;
            session.poll(&scope).await;
            assert!(!session.file.path().exists());
            std::fs::write(session.file.path(), &original)?;
            session.poll(&scope).await;
            assert!(session.pending_test_status.is_none());
            assert!(session.io_error.is_none());
            assert!(session.file.read()?.unwrap().contains("Space pressed 1"));
            Ok(())
        })
        .await
}
