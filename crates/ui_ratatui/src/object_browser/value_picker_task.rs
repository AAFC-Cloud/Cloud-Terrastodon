use super::pickers::FieldValuePicker;
use super::pickers::ValuePickerChoice;
use crate::object_explorer::SlotId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalCoordinator;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(super) enum ValuePickerOutcome {
    Selected {
        destination: SlotId,
        field: usize,
        choice: ValuePickerChoice,
    },
    Cancelled,
    Failed(String),
}

pub(super) struct ValuePickerTask {
    handle: JoinHandle<eyre::Result<ValuePickerOutcome>>,
    cancellation: CancellationToken,
}

impl ValuePickerTask {
    pub(super) fn spawn(
        picker: &FieldValuePicker,
        cancellation: &CancellationScope,
    ) -> Result<Self, String> {
        let choices = picker
            .rows()
            .iter()
            .map(|row| Choice {
                key: row.label().to_owned(),
                value: row.choice().clone(),
            })
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err("no compatible object choices are available".to_owned());
        }
        let destination = picker.destination();
        let field = picker.field();
        let header = format!("Pick Object for {}", picker.field_name());
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let (cancellation, future) = cancellation.with_async_child_scope(move |scope| async move {
            eyre::Ok(
                match PickerTui::new()
                    .set_header(header)
                    .set_auto_accept(false)
                    .terminal_coordinator(coordinator)
                    .pick_one(choices, &scope)
                    .await
                {
                    Ok(choice) => ValuePickerOutcome::Selected {
                        destination,
                        field,
                        choice,
                    },
                    Err(PickError::Cancelled) => ValuePickerOutcome::Cancelled,
                    Err(error) => ValuePickerOutcome::Failed(error.to_string()),
                },
            )
        });
        let handle = tokio::spawn(future);
        Ok(Self {
            handle,
            cancellation,
        })
    }

    pub(super) async fn close(self) {
        self.cancellation.request_cancel("object browser closed");
        match self.finish().await {
            Ok(ValuePickerOutcome::Failed(error)) | Err(error) => {
                tracing::warn!(%error, "Nested picker failed during shutdown");
            }
            Ok(_) => {}
        }
    }
    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<ValuePickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("value PickerTui task failed: {error}"))?
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn close_retains_required_work_until_the_task_has_finished() -> eyre::Result<()> {
        tokio::time::timeout(Duration::from_secs(5), async {
            CancellationToken::new()
                .with_async_scope(async |parent| {
                    let (started_tx, started_rx) = oneshot::channel();
                    let (release_tx, release_rx) = oneshot::channel();
                    let completed = Arc::new(AtomicBool::new(false));
                    let completed_in_task = Arc::clone(&completed);
                    let (cancellation, future) =
                        parent.with_async_child_scope(move |child| async move {
                            child
                                .deferred("finish the selected operation", async |_completion| {
                                    started_tx
                                        .send(())
                                        .expect("the test is waiting for work to start");
                                    release_rx.await?;
                                    completed_in_task.store(true, Ordering::SeqCst);
                                    eyre::Ok(())
                                })
                                .await?;
                            eyre::Ok(ValuePickerOutcome::Cancelled)
                        });
                    let notification = cancellation.clone();
                    let task = ValuePickerTask {
                        handle: tokio::spawn(future),
                        cancellation,
                    };
                    started_rx.await?;

                    let mut closing = Box::pin(task.close());
                    tokio::select! {
                        () = &mut closing => panic!("close abandoned admitted required work"),
                        () = notification.cancelled() => {}
                    }
                    assert!(!completed.load(Ordering::SeqCst));
                    assert!(!parent.is_cancelled());
                    release_tx.send(()).expect("required work remains alive");
                    closing.await;
                    assert!(completed.load(Ordering::SeqCst));
                    eyre::Ok(())
                })
                .await
        })
        .await?
    }
}
