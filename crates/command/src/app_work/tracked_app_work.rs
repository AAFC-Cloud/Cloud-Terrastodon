use super::WorkHandle;
use crate::CancellationReportExt;
use eyre::Report;
use eyre::Result;
use std::collections::HashMap;
use tokio::task::Id;
use tokio::task::JoinError;
use tokio::task::JoinSet;
use tracing::error;
use tracing::info;

/// Task ownership and observed failures for one background-work lifetime.
#[derive(Debug, Default)]
pub(super) struct TrackedAppWork {
    pub tasks: JoinSet<Result<()>>,
    pub handles: HashMap<Id, WorkHandle>,
    pub first_error: Option<Report>,
}

impl TrackedAppWork {
    pub fn record_completion(&mut self, result: Result<(Id, Result<()>), JoinError>) {
        let (id, result) = match result {
            Ok((id, result)) => (id, result),
            Err(error) => (error.id(), Err(Report::new(error))),
        };
        let description = self
            .handles
            .remove(&id)
            .map_or_else(|| format!("task {id}"), |handle| handle.description);
        if let Err(error) = result {
            if error.is_cancellation() {
                info!(%description, "Background work acknowledged cancellation");
                return;
            }
            let error = error.wrap_err(format!("Background work {description:?} failed"));
            error!(
                ?error,
                "Background work failed; remaining work will still be drained"
            );
            if self.first_error.is_none() {
                self.first_error = Some(error);
            }
        }
    }
}
