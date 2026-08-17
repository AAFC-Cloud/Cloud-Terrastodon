use super::arena_query_command::ArenaQueryCommand;
use super::arena_query_command::CommandResponse;
use super::arena_query_session::JsonBatch;
use super::arena_query_session::JsonBatchBudget;
use super::arena_query_session::QuerySessionEnd;
use super::arena_query_session::QuerySessionId;
use super::breadcrumbs::Breadcrumbs;
use super::browse_command::BrowseCommand;
use super::browse_session::BrowseSessionId;
use super::browse_session::CardWindowBudget;
use super::card_navigation::CardNavigation;
use super::card_window::CardWindow;
use super::explorer_command::ExplorerCommand;
use super::explorer_command::ExplorerHandle;
use super::explorer_command::ExplorerInbox;
use super::explorer_command::explorer_channel;
use super::query_progress::QueryProgress;
use super::value_address::ValueAddress;
use super::value_candidate_window::ValueCandidateWindow;
use super::value_candidate_window::ValueCandidateWindowBudget;
use facet::Shape;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

tokio::task_local! {
    static CURRENT_ARENA_QUERY_CONTEXT: ArenaQueryContext;
}

#[derive(Clone, Debug)]
pub(crate) struct ArenaQueryContext {
    commands: mpsc::Sender<ExplorerCommand>,
    identity: Arc<()>,
}

/// Cancellation-safe client ownership of one engine-side export session.
///
/// Dropping the lease sender (including when its task is aborted) wakes the
/// engine's cancellation branch. Cleanup therefore does not depend on an
/// async Drop implementation or spare capacity in the bounded command queue.
#[derive(Debug)]
pub(crate) struct ArenaQuerySession {
    context: ArenaQueryContext,
    id: QuerySessionId,
    cancellation: Option<oneshot::Sender<()>>,
}

/// Cancellation-safe ownership of the UI's active lazy browse cursor.
#[derive(Debug)]
pub(crate) struct ArenaBrowseSession {
    context: ArenaQueryContext,
    id: BrowseSessionId,
    cancellation: Option<oneshot::Sender<()>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArenaQueryContextError {
    Missing,
    EngineStopped,
    ResponseDropped,
    Rejected(String),
}

impl fmt::Display for ArenaQueryContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(formatter, "no ArenaQueryContext is installed"),
            Self::EngineStopped => write!(formatter, "the explorer engine has stopped"),
            Self::ResponseDropped => {
                write!(formatter, "the explorer engine dropped a query response")
            }
            Self::Rejected(message) => write!(
                formatter,
                "the explorer engine rejected the query: {message}"
            ),
        }
    }
}

impl Error for ArenaQueryContextError {}

impl ArenaQueryContext {
    pub(crate) fn channel(capacity: usize) -> (Self, ExplorerInbox) {
        let (commands, inbox) = explorer_channel(capacity);
        (
            Self {
                commands,
                identity: Arc::new(()),
            },
            inbox,
        )
    }

    pub(crate) fn engine_handle(&self) -> ExplorerHandle {
        ExplorerHandle::from_sender(self.commands.clone())
    }

    pub(crate) fn try_current() -> Option<Self> {
        CURRENT_ARENA_QUERY_CONTEXT.try_with(Clone::clone).ok()
    }

    pub(crate) fn current() -> Result<Self, ArenaQueryContextError> {
        Self::try_current().ok_or(ArenaQueryContextError::Missing)
    }

    pub(crate) fn scope<'a, F>(&'a self, future: F) -> impl Future<Output = F::Output> + 'a
    where
        F: Future + 'a,
    {
        CURRENT_ARENA_QUERY_CONTEXT.scope(self.clone(), future)
    }

    pub(crate) async fn open_export(
        &self,
        breadcrumbs: Breadcrumbs,
    ) -> Result<ArenaQuerySession, ArenaQueryContextError> {
        let (cancellation, cancelled) = oneshot::channel();
        let (response, receiver) = oneshot::channel();
        self.send(ArenaQueryCommand::BeginExport {
            breadcrumbs,
            cancelled,
            response,
        })
        .await?;
        let id = Self::receive(receiver).await?;
        Ok(ArenaQuerySession {
            context: self.clone(),
            id,
            cancellation: Some(cancellation),
        })
    }

    pub(crate) async fn open_browse(
        &self,
        breadcrumbs: Breadcrumbs,
    ) -> Result<ArenaBrowseSession, ArenaQueryContextError> {
        let (cancellation, cancelled) = oneshot::channel();
        let (response, receiver) = oneshot::channel();
        self.send_browse(BrowseCommand::Begin {
            breadcrumbs,
            cancelled,
            response,
        })
        .await?;
        let id = Self::receive(receiver).await?;
        Ok(ArenaBrowseSession {
            context: self.clone(),
            id,
            cancellation: Some(cancellation),
        })
    }

    async fn next_json_batch(
        &self,
        session: QuerySessionId,
        budget: JsonBatchBudget,
    ) -> Result<JsonBatch, ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.send(ArenaQueryCommand::NextJsonBatch {
            session,
            max_work: budget.max_work(),
            max_bytes: budget.max_bytes(),
            response,
        })
        .await?;
        Self::receive(receiver).await
    }

    async fn end_export(
        &self,
        session: QuerySessionId,
        end: QuerySessionEnd,
    ) -> Result<(), ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.send(ArenaQueryCommand::EndExport {
            session,
            end,
            response,
        })
        .await?;
        Self::receive(receiver).await
    }

    async fn send(&self, command: ArenaQueryCommand) -> Result<(), ArenaQueryContextError> {
        self.commands
            .send(ExplorerCommand::Query(command))
            .await
            .map_err(|_| ArenaQueryContextError::EngineStopped)
    }

    async fn send_browse(&self, command: BrowseCommand) -> Result<(), ArenaQueryContextError> {
        self.commands
            .send(ExplorerCommand::Browse(command))
            .await
            .map_err(|_| ArenaQueryContextError::EngineStopped)
    }

    async fn receive<T>(
        receiver: oneshot::Receiver<CommandResponse<T>>,
    ) -> Result<T, ArenaQueryContextError> {
        receiver
            .await
            .map_err(|_| ArenaQueryContextError::ResponseDropped)?
            .map_err(ArenaQueryContextError::Rejected)
    }

    #[cfg(test)]
    fn is_same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
}

impl ArenaBrowseSession {
    pub(crate) const fn id(&self) -> BrowseSessionId {
        self.id
    }

    pub(crate) async fn set_query(
        &self,
        breadcrumbs: Breadcrumbs,
    ) -> Result<(), ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::SetQuery {
                session: self.id,
                breadcrumbs,
                response,
            })
            .await?;
        ArenaQueryContext::receive(receiver).await
    }

    pub(crate) async fn fill_card_window(
        &self,
        anchor: Option<ValueAddress>,
        budget: CardWindowBudget,
    ) -> Result<QueryProgress<CardWindow>, ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::FillCardWindow {
                session: self.id,
                anchor,
                max_work: budget.max_work(),
                max_cards: budget.max_cards().get(),
                max_relationship_rows: budget.max_relationship_rows(),
                response,
            })
            .await?;
        ArenaQueryContext::receive(receiver).await
    }

    pub(crate) async fn navigate(
        &self,
        from: ValueAddress,
        direction: CardNavigation,
        max_work: usize,
    ) -> Result<QueryProgress<ValueAddress>, ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::Navigate {
                session: self.id,
                from,
                direction,
                max_work,
                response,
            })
            .await?;
        ArenaQueryContext::receive(receiver).await
    }

    pub(crate) async fn set_candidate_shape(
        &self,
        target_shape: &'static Shape,
    ) -> Result<(), ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::SetCandidateShape {
                session: self.id,
                target_shape,
                response,
            })
            .await?;
        ArenaQueryContext::receive(receiver).await
    }

    pub(crate) async fn fill_value_candidates(
        &self,
        anchor: Option<ValueAddress>,
        budget: ValueCandidateWindowBudget,
    ) -> Result<QueryProgress<ValueCandidateWindow>, ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::FillValueCandidates {
                session: self.id,
                anchor,
                max_work: budget.max_work(),
                max_candidates: budget.max_candidates().get(),
                response,
            })
            .await?;
        ArenaQueryContext::receive(receiver).await
    }

    pub(crate) async fn clear_value_candidates(&self) -> Result<(), ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::ClearValueCandidates {
                session: self.id,
                response,
            })
            .await?;
        ArenaQueryContext::receive(receiver).await
    }

    pub(crate) async fn close(mut self) -> Result<(), ArenaQueryContextError> {
        let (response, receiver) = oneshot::channel();
        self.context
            .send_browse(BrowseCommand::End {
                session: self.id,
                response,
            })
            .await?;
        let result = ArenaQueryContext::receive(receiver).await;
        if result.is_ok() {
            self.cancellation.take();
        }
        result
    }
}

impl Drop for ArenaBrowseSession {
    fn drop(&mut self) {
        // As with exports, closing the sender is a synchronous cancellation
        // signal that also runs when the owning UI future is aborted.
        self.cancellation.take();
    }
}

impl ArenaQuerySession {
    pub(crate) const fn id(&self) -> QuerySessionId {
        self.id
    }

    pub(crate) async fn next_json_batch(
        &self,
        budget: JsonBatchBudget,
    ) -> Result<JsonBatch, ArenaQueryContextError> {
        self.context.next_json_batch(self.id, budget).await
    }

    pub(crate) async fn complete(self) -> Result<(), ArenaQueryContextError> {
        self.close(QuerySessionEnd::Complete).await
    }

    pub(crate) async fn cancel(self) -> Result<(), ArenaQueryContextError> {
        self.close(QuerySessionEnd::Cancelled).await
    }

    async fn close(mut self, end: QuerySessionEnd) -> Result<(), ArenaQueryContextError> {
        let result = self.context.end_export(self.id, end).await;
        if result.is_ok() {
            // The engine has already closed the session. Disarming by dropping
            // this sender is harmless because its receiver was dropped with
            // the completed engine-side session.
            self.cancellation.take();
        }
        result
    }
}

impl Drop for ArenaQuerySession {
    fn drop(&mut self) {
        // Dropping the sender is the cancellation signal. This runs on task
        // abort as well as ordinary early return.
        self.cancellation.take();
    }
}

pub(crate) trait ArenaQueryContextFutureExt: Future + Sized {
    fn with_arena_query_context(
        self,
        context: ArenaQueryContext,
    ) -> impl Future<Output = Self::Output>;
}

impl<F> ArenaQueryContextFutureExt for F
where
    F: Future + Sized,
{
    fn with_arena_query_context(
        self,
        context: ArenaQueryContext,
    ) -> impl Future<Output = Self::Output> {
        CURRENT_ARENA_QUERY_CONTEXT.scope(context, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena_query_command::ArenaQueryCommand;
    use crate::object_explorer::arena_query_session::JsonBatch;
    use crate::object_explorer::arena_query_session::JsonBatchBudget;
    use crate::object_explorer::arena_query_session::QuerySessionEnd;
    use crate::object_explorer::arena_query_session::QuerySessionId;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::explorer_command::ExplorerCommand;

    #[tokio::test]
    async fn arena_query_context_scope_is_nested_and_restored() {
        let (outer, _outer_inbox) = ArenaQueryContext::channel(1);
        let (inner, _inner_inbox) = ArenaQueryContext::channel(1);
        assert!(ArenaQueryContext::try_current().is_none());

        outer
            .scope(async {
                assert!(
                    ArenaQueryContext::current()
                        .expect("outer context")
                        .is_same(&outer)
                );
                inner
                    .scope(async {
                        assert!(
                            ArenaQueryContext::current()
                                .expect("inner context")
                                .is_same(&inner)
                        );
                    })
                    .await;
                assert!(
                    ArenaQueryContext::current()
                        .expect("outer context restored")
                        .is_same(&outer)
                );
            })
            .await;

        assert!(ArenaQueryContext::try_current().is_none());
    }

    #[tokio::test]
    async fn arena_query_context_raw_spawn_requires_explicit_attachment() {
        let (context, _inbox) = ArenaQueryContext::channel(1);

        let raw_spawn_missing = context
            .scope(async {
                tokio::spawn(async {
                    matches!(
                        ArenaQueryContext::current(),
                        Err(ArenaQueryContextError::Missing)
                    )
                })
                .await
                .expect("raw task completes")
            })
            .await;
        assert!(raw_spawn_missing);

        let attached = tokio::spawn(
            async { ArenaQueryContext::current().is_ok() }
                .with_arena_query_context(context.clone()),
        )
        .await
        .expect("attached task completes");
        assert!(attached);
    }

    #[tokio::test]
    async fn arena_query_context_command_protocol_is_bounded_and_value_free() {
        let (context, mut inbox) = ArenaQueryContext::channel(1);
        let breadcrumbs = Breadcrumbs::default();
        let expected_breadcrumbs = breadcrumbs.clone();

        let engine = tokio::spawn(async move {
            let Some(ExplorerCommand::Query(ArenaQueryCommand::BeginExport {
                breadcrumbs,
                cancelled,
                response,
            })) = inbox.recv().await
            else {
                panic!("expected BeginExport");
            };
            assert_eq!(breadcrumbs, expected_breadcrumbs);
            response
                .send(Ok(QuerySessionId::new(3)))
                .expect("requester retained begin response");

            let Some(ExplorerCommand::Query(ArenaQueryCommand::NextJsonBatch {
                session,
                max_work,
                max_bytes,
                response,
            })) = inbox.recv().await
            else {
                panic!("expected NextJsonBatch");
            };
            assert_eq!(session.get(), 3);
            assert_eq!(max_work, 8);
            assert_eq!(max_bytes, 1_024);
            response
                .send(Ok(JsonBatch {
                    fragment: "{\"name\":\"Ada\"}".to_owned(),
                    inspected: 8,
                    emitted: 1,
                    complete: true,
                }))
                .expect("requester retained batch response");

            let Some(ExplorerCommand::Query(ArenaQueryCommand::EndExport {
                session,
                end,
                response,
            })) = inbox.recv().await
            else {
                panic!("expected EndExport");
            };
            assert_eq!(session.get(), 3);
            assert_eq!(end, QuerySessionEnd::Complete);
            response
                .send(Ok(()))
                .expect("requester retained end response");
            drop(cancelled);
        });

        let session = context
            .open_export(breadcrumbs)
            .await
            .expect("begin accepted");
        assert_eq!(session.id().get(), 3);
        let batch = session
            .next_json_batch(JsonBatchBudget::new(8, 1_024))
            .await
            .expect("batch accepted");
        assert_eq!(batch.fragment, "{\"name\":\"Ada\"}");
        assert_eq!(batch.inspected, 8);
        assert_eq!(batch.emitted, 1);
        assert!(batch.complete);
        session.complete().await.expect("end accepted");
        engine.await.expect("fake engine completes");
    }

    #[tokio::test]
    async fn arena_query_context_reports_engine_shutdown() {
        let (context, inbox) = ArenaQueryContext::channel(1);
        drop(inbox);

        let error = context
            .open_export(Breadcrumbs::default())
            .await
            .expect_err("closed engine must be reported");

        assert_eq!(error, ArenaQueryContextError::EngineStopped);
    }

    #[tokio::test]
    async fn arena_query_context_can_cancel_an_export_session() {
        let (context, mut inbox) = ArenaQueryContext::channel(1);
        let breadcrumbs = Breadcrumbs::default();
        let engine = tokio::spawn(async move {
            let Some(ExplorerCommand::Query(ArenaQueryCommand::BeginExport {
                cancelled,
                response,
                ..
            })) = inbox.recv().await
            else {
                panic!("expected BeginExport");
            };
            response
                .send(Ok(QuerySessionId::new(19)))
                .expect("requester retained begin response");

            let Some(ExplorerCommand::Query(ArenaQueryCommand::EndExport {
                session,
                end,
                response,
            })) = inbox.recv().await
            else {
                panic!("expected EndExport cancellation");
            };
            assert_eq!(session.get(), 19);
            assert_eq!(end, QuerySessionEnd::Cancelled);
            response
                .send(Ok(()))
                .expect("requester retained cancellation response");
            drop(cancelled);
        });

        context
            .open_export(breadcrumbs)
            .await
            .expect("begin accepted")
            .cancel()
            .await
            .expect("cancellation accepted");
        engine.await.expect("fake engine completes");
    }

    #[tokio::test]
    async fn dropping_query_session_closes_its_cancellation_lease() {
        let (context, mut inbox) = ArenaQueryContext::channel(1);
        let engine = tokio::spawn(async move {
            let Some(ExplorerCommand::Query(ArenaQueryCommand::BeginExport {
                cancelled,
                response,
                ..
            })) = inbox.recv().await
            else {
                panic!("expected BeginExport");
            };
            response
                .send(Ok(QuerySessionId::new(23)))
                .expect("requester retained begin response");
            assert!(
                cancelled.await.is_err(),
                "dropping the lease sender signals cancellation"
            );
        });

        let session = context
            .open_export(Breadcrumbs::default())
            .await
            .expect("begin accepted");
        drop(session);

        engine.await.expect("fake engine completes");
    }
}
