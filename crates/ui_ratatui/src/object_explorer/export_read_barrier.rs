use super::arena_query_session::QuerySessionId;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BarrierError {
    AlreadyActive(QuerySessionId),
    NotActive,
    WrongSession {
        active: QuerySessionId,
        requested: QuerySessionId,
    },
}

impl fmt::Display for BarrierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyActive(session) => {
                write!(
                    formatter,
                    "export session {} is already active",
                    session.get()
                )
            }
            Self::NotActive => write!(formatter, "no export session is active"),
            Self::WrongSession { active, requested } => write!(
                formatter,
                "export session {} is active, not {}",
                active.get(),
                requested.get()
            ),
        }
    }
}

impl Error for BarrierError {}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MutationSubmission<C> {
    ApplyNow(C),
    Deferred,
}

/// Arena-wide mutation/ingestion barrier for one coherent export.
///
/// Read-only UI work and bounded query reads remain outside this type. Commands
/// that would change Arena are retained in their engine-observed order until
/// the owning export completes or is cancelled.
pub(crate) struct ExportReadBarrier<C> {
    active: Option<QuerySessionId>,
    deferred: VecDeque<C>,
}

impl<C> Default for ExportReadBarrier<C> {
    fn default() -> Self {
        Self {
            active: None,
            deferred: VecDeque::new(),
        }
    }
}

impl<C> ExportReadBarrier<C> {
    pub(crate) fn begin(&mut self, session: QuerySessionId) -> Result<(), BarrierError> {
        if let Some(active) = self.active {
            return Err(BarrierError::AlreadyActive(active));
        }
        self.active = Some(session);
        Ok(())
    }

    pub(crate) fn submit(&mut self, command: C) -> MutationSubmission<C> {
        if self.active.is_some() {
            self.deferred.push_back(command);
            MutationSubmission::Deferred
        } else {
            MutationSubmission::ApplyNow(command)
        }
    }

    pub(crate) fn finish(&mut self, session: QuerySessionId) -> Result<VecDeque<C>, BarrierError> {
        self.assert_session(session)?;
        self.active = None;
        Ok(std::mem::take(&mut self.deferred))
    }

    pub(crate) fn cancel(&mut self, session: QuerySessionId) -> Result<VecDeque<C>, BarrierError> {
        self.finish(session)
    }

    pub(crate) const fn active_session(&self) -> Option<QuerySessionId> {
        self.active
    }

    pub(crate) fn deferred_len(&self) -> usize {
        self.deferred.len()
    }

    fn assert_session(&self, requested: QuerySessionId) -> Result<(), BarrierError> {
        match self.active {
            Some(active) if active == requested => Ok(()),
            Some(active) => Err(BarrierError::WrongSession { active, requested }),
            None => Err(BarrierError::NotActive),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Mutation {
        Ui(&'static str),
        Completion(&'static str),
    }

    #[test]
    fn export_read_barrier_defers_mutations_until_close() {
        let session = QuerySessionId::new(5);
        let mut barrier = ExportReadBarrier::default();

        assert_eq!(
            barrier.submit(Mutation::Ui("before")),
            MutationSubmission::ApplyNow(Mutation::Ui("before"))
        );
        barrier.begin(session).unwrap();
        assert_eq!(
            barrier.submit(Mutation::Ui("during-1")),
            MutationSubmission::Deferred
        );
        assert_eq!(
            barrier.submit(Mutation::Completion("during-2")),
            MutationSubmission::Deferred
        );
        assert_eq!(barrier.active_session(), Some(session));
        assert_eq!(barrier.deferred_len(), 2);

        assert_eq!(
            barrier.finish(session).unwrap(),
            [Mutation::Ui("during-1"), Mutation::Completion("during-2"),]
        );
        assert_eq!(barrier.active_session(), None);
        assert_eq!(
            barrier.submit(Mutation::Ui("after")),
            MutationSubmission::ApplyNow(Mutation::Ui("after"))
        );
    }

    #[test]
    fn cancelling_export_releases_deferred_commands_in_observed_order() {
        let session = QuerySessionId::new(8);
        let mut barrier = ExportReadBarrier::default();
        barrier.begin(session).unwrap();
        barrier.submit(Mutation::Completion("first"));
        barrier.submit(Mutation::Ui("second"));

        assert_eq!(
            barrier.cancel(session).unwrap(),
            [Mutation::Completion("first"), Mutation::Ui("second"),]
        );
        assert_eq!(barrier.active_session(), None);
    }

    #[tokio::test]
    async fn background_invocations_run_while_export_ingestion_is_deferred() {
        let session = QuerySessionId::new(13);
        let mut barrier = ExportReadBarrier::default();
        barrier.begin(session).unwrap();
        let background = tokio::spawn(async { Mutation::Completion("network result ready") });

        let completion = background.await.expect("background future completes");
        assert_eq!(
            barrier.submit(completion),
            MutationSubmission::Deferred,
            "only engine ingestion is deferred; the future itself completes"
        );
        assert_eq!(
            barrier.finish(session).unwrap(),
            [Mutation::Completion("network result ready")]
        );
    }

    #[test]
    fn wrong_export_session_cannot_release_the_barrier() {
        let active = QuerySessionId::new(21);
        let requested = QuerySessionId::new(22);
        let mut barrier = ExportReadBarrier::<Mutation>::default();
        barrier.begin(active).unwrap();

        assert_eq!(
            barrier.finish(requested),
            Err(BarrierError::WrongSession { active, requested })
        );
        assert_eq!(barrier.active_session(), Some(active));
    }
}
