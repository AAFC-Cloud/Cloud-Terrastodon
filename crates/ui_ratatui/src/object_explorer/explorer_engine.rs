use std::num::NonZeroUsize;

use cloud_terrastodon_registry::RuntimeValue;
use facet::Facet;
use tokio::sync::oneshot;

use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::arena_query_command::{ArenaQueryCommand, CommandResponse};
use super::arena_query_session::{QuerySessionEnd, QuerySessionId};
use super::borrow_graph::BorrowGraph;
use super::breadcrumb::Breadcrumb;
use super::breadcrumbs::Breadcrumbs;
use super::browse_command::BrowseCommand;
use super::browse_session::BrowseSessionId;
use super::card_window::CardWindow;
use super::explorer_command::{
    ArenaMutationCommand, ArenaReadCommand, ExplorerCommand, ExplorerInbox,
};
use super::export_read_barrier::{ExportReadBarrier, MutationSubmission};
use super::invocation_controller::InvocationController;
use super::invocation_host::InvocationHost;
use super::json_encoder::JsonEncoder;
use super::json_export_job::JsonExportJob;
#[cfg(test)]
use super::preorder_cursor::PreorderCursor;
use super::production_controller::ProductionController;
use super::query_cursor::QueryCursor;
use super::query_plan::QueryPlan;
use super::query_progress::{QueryProgress, QueryProgressState};
use super::revision::{QueryRevision, ScanRevisionStamp};
use super::tab::Tab;
use super::tokio_invocation_host::TokioInvocationHost;
use super::value_builder::BuilderStore;
use super::value_candidate::ValueCandidate;
use super::value_candidate_window::ValueCandidateWindow;
use super::work_budget::WorkBudget;

/// Headless single-owner state machine for the reflected object explorer.
///
/// Background work may run concurrently, but Arena state changes only when a
/// mutation command reaches this FIFO inbox. During an export, the engine
/// keeps servicing bounded reads and query batches while retaining mutation
/// commands in the export barrier's observed order.
pub(crate) struct ExplorerEngine {
    arena: Arena,
    next_query_session: u64,
    next_browse_session: u64,
    export_barrier: ExportReadBarrier<ArenaMutationCommand>,
    json_encoder: JsonEncoder,
    builders: BuilderStore,
    borrow_graph: BorrowGraph,
    invocations: InvocationController,
    productions: ProductionController,
    invocation_host: Box<dyn InvocationHost>,
}

impl Drop for ExplorerEngine {
    fn drop(&mut self) {
        // No command can interleave with engine destruction. Remove every
        // graph edge before Arena values are dropped so no lease metadata
        // survives its owning session.
        self.builders.release_all_leases(&mut self.borrow_graph);
    }
}

impl ExplorerEngine {
    pub(crate) fn empty() -> Self {
        Self::new(Arena::default())
    }

    pub(crate) fn new(arena: Arena) -> Self {
        Self::with_invocation_host(
            arena,
            Box::new(TokioInvocationHost::new(identity_invocation_future)),
        )
    }

    pub(crate) fn empty_with_invocation_host(host: Box<dyn InvocationHost>) -> Self {
        Self::with_invocation_host(Arena::default(), host)
    }

    pub(crate) fn empty_with_tokio_invocation_host(
        attach: fn(
            cloud_terrastodon_registry::InvocationFuture,
        ) -> cloud_terrastodon_registry::InvocationFuture,
    ) -> Self {
        Self::with_invocation_host(Arena::default(), Box::new(TokioInvocationHost::new(attach)))
    }

    fn with_invocation_host(arena: Arena, invocation_host: Box<dyn InvocationHost>) -> Self {
        Self {
            arena,
            next_query_session: 1,
            next_browse_session: 1,
            export_barrier: ExportReadBarrier::default(),
            json_encoder: JsonEncoder::default(),
            builders: BuilderStore::default(),
            borrow_graph: BorrowGraph::default(),
            invocations: InvocationController::default(),
            productions: ProductionController::default(),
            invocation_host,
        }
    }

    pub(crate) const fn arena(&self) -> &Arena {
        &self.arena
    }

    pub(crate) const fn json_serialization_count(&self) -> usize {
        self.json_encoder.encoded_values()
    }

    pub(crate) const fn builders(&self) -> &BuilderStore {
        &self.builders
    }

    pub(crate) const fn borrow_graph(&self) -> &BorrowGraph {
        &self.borrow_graph
    }

    pub(crate) fn pending_invocation_count(&self) -> usize {
        self.invocations.pending_count()
    }

    pub(crate) fn invocation_plan_count(&self) -> usize {
        self.invocations.plan_count()
    }

    pub(crate) fn active_production_count(&self) -> usize {
        self.productions.active_count()
    }

    /// Drive the engine until every command handle has been dropped.
    ///
    /// This future deliberately need not be spawned: the Ratatui adapter can
    /// poll it as part of its owning event loop, while Send producer futures
    /// communicate through ArenaQueryContext.
    pub(crate) async fn run(mut self, mut inbox: ExplorerInbox) -> Self {
        while let Some(command) = inbox.recv().await {
            match command {
                ExplorerCommand::Mutation(command) => {
                    apply_mutation(
                        &mut self.arena,
                        &mut self.builders,
                        &mut self.borrow_graph,
                        &mut self.invocations,
                        &mut self.productions,
                        self.invocation_host.as_mut(),
                        command,
                    );
                }
                ExplorerCommand::Read(command) => {
                    apply_read(
                        &self.arena,
                        &self.builders,
                        &self.borrow_graph,
                        &mut self.json_encoder,
                        command,
                    );
                }
                ExplorerCommand::Query(ArenaQueryCommand::BeginExport {
                    breadcrumbs,
                    cancelled,
                    response,
                }) => {
                    let inbox_closed = begin_and_serve_export(
                        &mut self.arena,
                        &mut self.builders,
                        &mut self.borrow_graph,
                        &mut self.invocations,
                        &mut self.productions,
                        self.invocation_host.as_mut(),
                        &mut self.export_barrier,
                        &mut self.json_encoder,
                        &mut self.next_query_session,
                        &mut inbox,
                        breadcrumbs,
                        cancelled,
                        response,
                    )
                    .await;
                    if inbox_closed {
                        break;
                    }
                }
                ExplorerCommand::Query(command) => {
                    reject_inactive_query(command);
                }
                ExplorerCommand::Browse(BrowseCommand::Begin {
                    breadcrumbs,
                    cancelled,
                    response,
                }) => {
                    let session = self.allocate_browse_session();
                    if response.send(Ok(session)).is_err() {
                        continue;
                    }
                    let inbox_closed = serve_browse(
                        &mut self.arena,
                        &mut self.builders,
                        &mut self.borrow_graph,
                        &mut self.invocations,
                        &mut self.productions,
                        self.invocation_host.as_mut(),
                        &mut self.export_barrier,
                        &mut self.json_encoder,
                        &mut self.next_query_session,
                        &mut inbox,
                        breadcrumbs,
                        session,
                        cancelled,
                    )
                    .await;
                    if inbox_closed {
                        break;
                    }
                }
                ExplorerCommand::Browse(command) => reject_inactive_browse(command),
            }
        }
        self
    }

    fn allocate_query_session(&mut self) -> QuerySessionId {
        allocate_query_session(&mut self.next_query_session)
    }

    fn allocate_browse_session(&mut self) -> BrowseSessionId {
        let session = BrowseSessionId::new(self.next_browse_session);
        self.next_browse_session = self
            .next_browse_session
            .checked_add(1)
            .expect("browse session id space exhausted");
        session
    }
}

fn identity_invocation_future(
    future: cloud_terrastodon_registry::InvocationFuture,
) -> cloud_terrastodon_registry::InvocationFuture {
    future
}

fn allocate_query_session(next_query_session: &mut u64) -> QuerySessionId {
    let session = QuerySessionId::new(*next_query_session);
    *next_query_session = next_query_session
        .checked_add(1)
        .expect("query session id space exhausted");
    session
}

async fn begin_and_serve_export(
    arena: &mut Arena,
    builders: &mut BuilderStore,
    borrow_graph: &mut BorrowGraph,
    invocations: &mut InvocationController,
    productions: &mut ProductionController,
    invocation_host: &mut dyn InvocationHost,
    barrier: &mut ExportReadBarrier<ArenaMutationCommand>,
    json_encoder: &mut JsonEncoder,
    next_query_session: &mut u64,
    inbox: &mut ExplorerInbox,
    breadcrumbs: Breadcrumbs,
    cancelled: oneshot::Receiver<()>,
    response: oneshot::Sender<CommandResponse<QuerySessionId>>,
) -> bool {
    let session = allocate_query_session(next_query_session);
    barrier
        .begin(session)
        .expect("the single-owner engine starts only one export at a time");
    if response.send(Ok(session)).is_err() {
        let deferred = barrier
            .cancel(session)
            .expect("the abandoned opener still owns the export barrier");
        apply_deferred(
            arena,
            builders,
            borrow_graph,
            invocations,
            productions,
            invocation_host,
            deferred,
        );
        return false;
    }

    serve_export(
        arena,
        builders,
        borrow_graph,
        invocations,
        productions,
        invocation_host,
        barrier,
        json_encoder,
        inbox,
        QueryPlan::new(breadcrumbs),
        session,
        cancelled,
    )
    .await
}

enum ExportExit {
    End {
        end: QuerySessionEnd,
        response: oneshot::Sender<CommandResponse<()>>,
    },
    LeaseDropped,
    InboxClosed,
}

async fn serve_export(
    arena: &mut Arena,
    builders: &mut BuilderStore,
    borrow_graph: &mut BorrowGraph,
    invocations: &mut InvocationController,
    productions: &mut ProductionController,
    invocation_host: &mut dyn InvocationHost,
    barrier: &mut ExportReadBarrier<ArenaMutationCommand>,
    json_encoder: &mut JsonEncoder,
    inbox: &mut ExplorerInbox,
    query_plan: QueryPlan,
    session: QuerySessionId,
    mut cancelled: oneshot::Receiver<()>,
) -> bool {
    let arena_revision = arena.arena_revision();
    let source = ArenaAddressSource::new(arena);
    let mut export = JsonExportJob::for_arena(&source, query_plan, arena_revision);

    let exit = loop {
        tokio::select! {
            biased;
            _ = &mut cancelled => break ExportExit::LeaseDropped,
            command = inbox.recv() => {
                let Some(command) = command else {
                    break ExportExit::InboxClosed;
                };
                match command {
                    ExplorerCommand::Mutation(command) => {
                        debug_assert!(matches!(
                            barrier.submit(command),
                            MutationSubmission::Deferred
                        ));
                    }
                    ExplorerCommand::Read(command) => {
                        apply_read_with_source(
                            arena,
                            &source,
                            builders,
                            borrow_graph,
                            json_encoder,
                            command,
                        )
                    }
                    ExplorerCommand::Query(ArenaQueryCommand::NextJsonBatch {
                        session: requested,
                        max_work,
                        max_bytes,
                        response,
                    }) if requested == session => {
                        let result = export.next_batch(json_encoder, max_work, max_bytes);
                        let _ = response.send(result);
                    }
                    ExplorerCommand::Query(ArenaQueryCommand::EndExport {
                        session: requested,
                        end,
                        response,
                    }) if requested == session => {
                        break ExportExit::End { end, response };
                    }
                    ExplorerCommand::Query(ArenaQueryCommand::BeginExport {
                        response,
                        ..
                    }) => {
                        let _ = response.send(Err(format!(
                            "export session {} is already active",
                            session.get()
                        )));
                    }
                    ExplorerCommand::Query(command) => reject_wrong_session(command, session),
                    ExplorerCommand::Browse(command) => reject_browse_during_export(command),
                }
            }
        }
    };

    // All Facet Peeks/iterators are gone before deferred mutation resumes.
    drop(export);
    drop(source);

    let deferred = match &exit {
        ExportExit::End {
            end: QuerySessionEnd::Complete,
            ..
        } => barrier.finish(session),
        ExportExit::End {
            end: QuerySessionEnd::Cancelled,
            ..
        }
        | ExportExit::LeaseDropped
        | ExportExit::InboxClosed => barrier.cancel(session),
    }
    .expect("active export owns its barrier");
    apply_deferred(
        arena,
        builders,
        borrow_graph,
        invocations,
        productions,
        invocation_host,
        deferred,
    );

    let inbox_closed = matches!(&exit, ExportExit::InboxClosed);
    if let ExportExit::End { response, .. } = exit {
        let _ = response.send(Ok(()));
    }

    inbox_closed
}

enum BrowseExit {
    Mutation(ArenaMutationCommand),
    BeginExport {
        breadcrumbs: Breadcrumbs,
        cancelled: oneshot::Receiver<()>,
        response: oneshot::Sender<CommandResponse<QuerySessionId>>,
    },
    SetQuery {
        breadcrumbs: Breadcrumbs,
        response: oneshot::Sender<CommandResponse<()>>,
    },
    SetCandidateShape {
        target_shape: &'static facet::Shape,
        response: oneshot::Sender<CommandResponse<()>>,
    },
    ClearValueCandidates {
        response: oneshot::Sender<CommandResponse<()>>,
    },
    End {
        response: oneshot::Sender<CommandResponse<()>>,
    },
    LeaseDropped,
    InboxClosed,
}

async fn serve_browse(
    arena: &mut Arena,
    builders: &mut BuilderStore,
    borrow_graph: &mut BorrowGraph,
    invocations: &mut InvocationController,
    productions: &mut ProductionController,
    invocation_host: &mut dyn InvocationHost,
    barrier: &mut ExportReadBarrier<ArenaMutationCommand>,
    json_encoder: &mut JsonEncoder,
    next_query_session: &mut u64,
    inbox: &mut ExplorerInbox,
    mut breadcrumbs: Breadcrumbs,
    session: BrowseSessionId,
    mut cancelled: oneshot::Receiver<()>,
) -> bool {
    const BROWSE_CACHE_CAPACITY: usize = 129;

    let mut query_revision = QueryRevision::default();
    let mut candidate_revision = QueryRevision::default();
    let mut candidate_shape = None;
    'rebuild: loop {
        let stamp = ScanRevisionStamp {
            arena: arena.arena_revision(),
            query: query_revision,
        };
        let source = ArenaAddressSource::object_pool(arena);
        let mut cursor = QueryCursor::new(
            &source,
            QueryPlan::new(breadcrumbs.clone()),
            stamp,
            NonZeroUsize::new(BROWSE_CACHE_CAPACITY).expect("non-zero browse cache capacity"),
        );
        let mut candidate_cursor = candidate_shape.map(|target_shape| {
            QueryCursor::new(
                &source,
                candidate_query_plan(target_shape),
                ScanRevisionStamp {
                    arena: arena.arena_revision(),
                    query: candidate_revision,
                },
                NonZeroUsize::new(BROWSE_CACHE_CAPACITY)
                    .expect("non-zero candidate cache capacity"),
            )
        });

        let exit = loop {
            tokio::select! {
                biased;
                _ = &mut cancelled => break BrowseExit::LeaseDropped,
                command = inbox.recv() => {
                    let Some(command) = command else {
                        break BrowseExit::InboxClosed;
                    };
                    match command {
                        ExplorerCommand::Mutation(ArenaMutationCommand::PollInvocations {
                            response,
                        }) if !invocations.has_ready(invocation_host) => {
                            let _ = response.send(Ok(Vec::new()));
                        }
                        ExplorerCommand::Mutation(command) => {
                            break BrowseExit::Mutation(command);
                        }
                        ExplorerCommand::Read(command) => {
                            apply_read_with_source(
                                arena,
                                &source,
                                builders,
                                borrow_graph,
                                json_encoder,
                                command,
                            );
                        }
                        ExplorerCommand::Query(ArenaQueryCommand::BeginExport {
                            breadcrumbs,
                            cancelled,
                            response,
                        }) => {
                            break BrowseExit::BeginExport {
                                breadcrumbs,
                                cancelled,
                                response,
                            };
                        }
                        ExplorerCommand::Query(command) => reject_inactive_query(command),
                        ExplorerCommand::Browse(BrowseCommand::SetQuery {
                            session: requested,
                            breadcrumbs,
                            response,
                        }) if requested == session => {
                            break BrowseExit::SetQuery {
                                breadcrumbs,
                                response,
                            };
                        }
                        ExplorerCommand::Browse(BrowseCommand::FillCardWindow {
                            session: requested,
                            anchor,
                            max_work,
                            max_cards,
                            max_relationship_rows,
                            response,
                        }) if requested == session => {
                            let result = NonZeroUsize::new(max_cards)
                                .ok_or_else(|| "a card window must request at least one card".to_owned())
                                .and_then(|max_cards| {
                                    if max_work == 0 {
                                        return Err("a card window must allow at least one unit of work".to_owned());
                                    }
                                    let mut work = WorkBudget::new(max_work);
                                    cursor
                                        .fill_window(anchor.as_ref(), max_cards, stamp, &mut work)
                                        .map_err(|error| error.to_string())
                                        .and_then(|progress| {
                                            observe_card_window_progress(
                                                arena,
                                                builders,
                                                &source,
                                                progress,
                                                max_relationship_rows,
                                            )
                                        })
                                });
                            let _ = response.send(result);
                        }
                        ExplorerCommand::Browse(BrowseCommand::Navigate {
                            session: requested,
                            from,
                            direction,
                            max_work,
                            response,
                        }) if requested == session => {
                            let result = if max_work == 0 {
                                Err("navigation must allow at least one unit of work".to_owned())
                            } else {
                                let mut work = WorkBudget::new(max_work);
                                Ok(cursor.adjacent_from(
                                    &from,
                                    direction,
                                    stamp,
                                    &mut work,
                                ))
                            };
                            let _ = response.send(result);
                        }
                        ExplorerCommand::Browse(BrowseCommand::SetCandidateShape {
                            session: requested,
                            target_shape,
                            response,
                        }) if requested == session => {
                            break BrowseExit::SetCandidateShape {
                                target_shape,
                                response,
                            };
                        }
                        ExplorerCommand::Browse(BrowseCommand::FillValueCandidates {
                            session: requested,
                            anchor,
                            max_work,
                            max_candidates,
                            response,
                        }) if requested == session => {
                            let result = candidate_cursor
                                .as_mut()
                                .ok_or_else(|| {
                                    "no value-candidate shape is active".to_owned()
                                })
                                .and_then(|candidate_cursor| {
                                    let max_candidates = NonZeroUsize::new(max_candidates)
                                        .ok_or_else(|| {
                                            "a candidate window must request at least one value"
                                                .to_owned()
                                        })?;
                                    if max_work == 0 {
                                        return Err(
                                            "a candidate window must allow at least one unit of work"
                                                .to_owned(),
                                        );
                                    }
                                    let mut work = WorkBudget::new(max_work);
                                    candidate_cursor
                                        .fill_window(
                                            anchor.as_ref(),
                                            max_candidates,
                                            ScanRevisionStamp {
                                                arena: arena.arena_revision(),
                                                query: candidate_revision,
                                            },
                                            &mut work,
                                        )
                                        .map_err(|error| error.to_string())
                                        .and_then(|progress| {
                                            observe_value_candidate_progress(&source, progress)
                                        })
                                });
                            let _ = response.send(result);
                        }
                        ExplorerCommand::Browse(BrowseCommand::ClearValueCandidates {
                            session: requested,
                            response,
                        }) if requested == session => {
                            break BrowseExit::ClearValueCandidates { response };
                        }
                        ExplorerCommand::Browse(BrowseCommand::End {
                            session: requested,
                            response,
                        }) if requested == session => {
                            break BrowseExit::End { response };
                        }
                        ExplorerCommand::Browse(BrowseCommand::Begin { response, .. }) => {
                            let _ = response.send(Err(format!(
                                "browse session {} is already active",
                                session.get()
                            )));
                        }
                        ExplorerCommand::Browse(command) => {
                            reject_wrong_browse_session(command, session);
                        }
                    }
                }
            }
        };

        // A cursor and its reflected source never survive an Arena mutation
        // or a coherent export barrier. The logical browse session does.
        drop(cursor);
        drop(candidate_cursor);
        drop(source);

        match exit {
            BrowseExit::Mutation(command) => {
                apply_mutation(
                    arena,
                    builders,
                    borrow_graph,
                    invocations,
                    productions,
                    invocation_host,
                    command,
                );
                continue 'rebuild;
            }
            BrowseExit::BeginExport {
                breadcrumbs: export_breadcrumbs,
                cancelled: export_cancelled,
                response,
            } => {
                if begin_and_serve_export(
                    arena,
                    builders,
                    borrow_graph,
                    invocations,
                    productions,
                    invocation_host,
                    barrier,
                    json_encoder,
                    next_query_session,
                    inbox,
                    export_breadcrumbs,
                    export_cancelled,
                    response,
                )
                .await
                {
                    return true;
                }
                continue 'rebuild;
            }
            BrowseExit::SetQuery {
                breadcrumbs: replacement,
                response,
            } => {
                breadcrumbs = replacement;
                query_revision = query_revision.next();
                let _ = response.send(Ok(()));
                continue 'rebuild;
            }
            BrowseExit::SetCandidateShape {
                target_shape,
                response,
            } => {
                candidate_shape = Some(target_shape);
                candidate_revision = candidate_revision.next();
                let _ = response.send(Ok(()));
                continue 'rebuild;
            }
            BrowseExit::ClearValueCandidates { response } => {
                candidate_shape = None;
                candidate_revision = candidate_revision.next();
                let _ = response.send(Ok(()));
                continue 'rebuild;
            }
            BrowseExit::End { response } => {
                let _ = response.send(Ok(()));
                return false;
            }
            BrowseExit::LeaseDropped => return false,
            BrowseExit::InboxClosed => return true,
        }
    }
}

fn candidate_query_plan(target_shape: &'static facet::Shape) -> QueryPlan {
    let mut included_shapes = vec![cloud_terrastodon_registry::describe_shape(target_shape)];
    if let facet::Def::Pointer(pointer) = target_shape.def
        && let Some(pointee) = pointer.pointee()
        && (cloud_terrastodon_registry::RuntimeValue::can_own_pointee(target_shape, pointee)
            || cloud_terrastodon_registry::RuntimeValue::can_borrow_pointee(target_shape, pointee))
    {
        let pointee = cloud_terrastodon_registry::describe_shape(pointee);
        if !included_shapes.contains(&pointee) {
            included_shapes.push(pointee);
        }
    }
    QueryPlan::new(Breadcrumbs::new(vec![Breadcrumb::ShapeFilter {
        included_shapes,
    }]))
}

fn observe_card_window_progress(
    arena: &Arena,
    builders: &BuilderStore,
    source: &ArenaAddressSource<'_>,
    progress: QueryProgress<super::query_window::QueryWindow>,
    max_relationship_rows: usize,
) -> CommandResponse<QueryProgress<CardWindow>> {
    let work_spent = progress.work_spent();
    let instrumentation = progress.instrumentation();
    let total = progress.total();
    let state = match progress.into_state() {
        QueryProgressState::Ready(window) => {
            let cards = window
                .addresses()
                .iter()
                .cloned()
                .map(|address| {
                    if address.path().segments().is_empty() && source.resolve(&address).is_err() {
                        super::root_snapshot::RootSnapshot::observe(
                            arena,
                            builders,
                            address.root_id(),
                            max_relationship_rows,
                        )
                        .map(|snapshot| snapshot.card().clone())
                        .map_err(|error| error.to_string())
                    } else {
                        super::card_snapshot::CardSnapshot::observe(
                            source,
                            address,
                            max_relationship_rows,
                        )
                        .map_err(|error| error.to_string())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            QueryProgressState::Ready(CardWindow::from_cards(
                cards,
                window.has_before(),
                window.has_after(),
            ))
        }
        QueryProgressState::Pending => QueryProgressState::Pending,
        QueryProgressState::Complete => QueryProgressState::Complete,
        QueryProgressState::Cancelled => QueryProgressState::Cancelled,
        QueryProgressState::Stale => QueryProgressState::Stale,
    };
    Ok(QueryProgress::new(
        state,
        work_spent,
        instrumentation,
        total,
    ))
}

fn observe_value_candidate_progress(
    source: &ArenaAddressSource<'_>,
    progress: QueryProgress<super::query_window::QueryWindow>,
) -> CommandResponse<QueryProgress<ValueCandidateWindow>> {
    let work_spent = progress.work_spent();
    let instrumentation = progress.instrumentation();
    let total = progress.total();
    let state = match progress.into_state() {
        QueryProgressState::Ready(window) => {
            let candidates = window
                .addresses()
                .iter()
                .cloned()
                .map(|address| {
                    ValueCandidate::resolve(source, address.clone())
                        .ok_or_else(|| format!("candidate address {address} no longer resolves"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            QueryProgressState::Ready(ValueCandidateWindow::new(
                candidates,
                window.has_before(),
                window.has_after(),
            ))
        }
        QueryProgressState::Pending => QueryProgressState::Pending,
        QueryProgressState::Complete => QueryProgressState::Complete,
        QueryProgressState::Cancelled => QueryProgressState::Cancelled,
        QueryProgressState::Stale => QueryProgressState::Stale,
    };
    Ok(QueryProgress::new(
        state,
        work_spent,
        instrumentation,
        total,
    ))
}

fn apply_mutation(
    arena: &mut Arena,
    builders: &mut BuilderStore,
    borrow_graph: &mut BorrowGraph,
    invocations: &mut InvocationController,
    productions: &mut ProductionController,
    invocation_host: &mut dyn InvocationHost,
    command: ArenaMutationCommand,
) {
    match command {
        ArenaMutationCommand::ReserveBuilder { response } => {
            let result = builders.reserve(arena).map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::SetBuilderShape {
            slot,
            shape,
            response,
        } => {
            let result = builders
                .set_shape_and_finalize(arena, borrow_graph, slot, shape)
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::CreateBuilder { shape, response } => {
            let result = builders
                .create_and_finalize(arena, borrow_graph, shape)
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::SetBuilderField {
            slot,
            field,
            binding,
            response,
        } => {
            let result = binding
                .into_binding()
                .map_err(|error| format!("could not ingest builder field value: {error}"))
                .and_then(|binding| {
                    builders
                        .set_field_and_finalize(arena, borrow_graph, slot, field, binding)
                        .map_err(|error| error.to_string())
                });
            let _ = response.send(result);
        }
        ArenaMutationCommand::UnsetBuilderField {
            slot,
            field,
            response,
        } => {
            let result = builders
                .unset_field_and_finalize(arena, borrow_graph, slot, field)
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::CompleteBuilderField {
            slot,
            field,
            binding,
            response,
        } => {
            let result = binding
                .into_binding()
                .map_err(|error| format!("could not ingest producer field value: {error}"))
                .and_then(|binding| {
                    builders
                        .complete_pending_field_and_finalize(
                            arena,
                            borrow_graph,
                            slot,
                            field,
                            binding,
                        )
                        .map_err(|error| error.to_string())
                });
            let _ = response.send(result);
        }
        ArenaMutationCommand::SelectBuilderVariant {
            slot,
            variant,
            response,
        } => {
            let result = builders
                .select_variant_and_finalize(arena, borrow_graph, slot, variant)
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::SetBuilderScalar {
            slot,
            value,
            response,
        } => {
            let result = value
                .into_runtime()
                .map_err(|error| format!("could not ingest builder scalar value: {error}"))
                .and_then(|value| {
                    builders
                        .set_scalar_and_finalize(arena, borrow_graph, slot, value)
                        .map_err(|error| error.to_string())
                });
            let _ = response.send(result);
        }
        ArenaMutationCommand::SetBuilderScalarText {
            slot,
            text,
            response,
        } => {
            let result = builders
                .builder(slot)
                .ok_or_else(|| format!("slot {slot} has no defined builder"))
                .and_then(|builder| {
                    cloud_terrastodon_registry::RuntimeValue::from_text(builder.shape(), &text)
                        .map_err(|error| format!("could not parse builder scalar: {error}"))
                })
                .and_then(|value| {
                    builders
                        .set_scalar_and_finalize(arena, borrow_graph, slot, value)
                        .map_err(|error| error.to_string())
                });
            let _ = response.send(result);
        }
        ArenaMutationCommand::Invoke {
            input,
            input_thing,
            function,
            mode,
            response,
        } => {
            let result = invocations
                .invoke(
                    arena,
                    builders,
                    borrow_graph,
                    invocation_host,
                    input,
                    input_thing,
                    function,
                    mode,
                )
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::InvokeArbitrary {
            request,
            request_function,
            constructor,
            bytes,
            response,
        } => {
            let result = invocations
                .invoke_arbitrary(
                    arena,
                    builders,
                    borrow_graph,
                    invocation_host,
                    request,
                    request_function,
                    constructor,
                    bytes,
                )
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::PollInvocations { response } => {
            let events = invocations.poll(arena, builders, borrow_graph, invocation_host);
            let _ = response.send(Ok(events));
        }
        ArenaMutationCommand::StartProduction {
            destination,
            field,
            function,
            strategy,
            max_work,
            response,
        } => {
            let result = productions.start(
                arena,
                builders,
                borrow_graph,
                invocations,
                invocation_host,
                destination,
                field,
                function,
                strategy,
                max_work,
            );
            let _ = response.send(result);
        }
        ArenaMutationCommand::AdvanceProductions { max_work, response } => {
            let result = productions.advance(
                arena,
                builders,
                borrow_graph,
                invocations,
                invocation_host,
                max_work,
            );
            let _ = response.send(result);
        }
        ArenaMutationCommand::UpdateTab {
            slot,
            update,
            response,
        } => {
            let result = borrow_graph
                .ensure_root_unprotected(slot)
                .map_err(|error| error.to_string())
                .and_then(|()| clone_tab(arena, slot))
                .and_then(|mut tab| {
                    tab.apply(update)?;
                    let value = RuntimeValue::from_box(Box::new(tab.clone()))
                        .map_err(|error| format!("could not reflect updated Tab: {error}"))?;
                    let previous = arena
                        .replace_ready(slot, value)
                        .map_err(|error| error.to_string())?;
                    drop(previous);
                    Ok(tab)
                });
            let _ = response.send(result);
        }
        ArenaMutationCommand::CancelInvocation {
            invocation,
            response,
        } => {
            let result = invocations
                .cancel(arena, builders, borrow_graph, invocation_host, invocation)
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaMutationCommand::InsertReady { value, response } => {
            let result = value
                .into_runtime()
                .map_err(|error| format!("could not ingest invocation output: {error}"))
                .and_then(|value| arena.insert_ready(value).map_err(|error| error.to_string()));
            let _ = response.send(result);
        }
        ArenaMutationCommand::SetReady {
            slot,
            value,
            response,
        } => {
            let result = value
                .into_runtime()
                .map_err(|error| format!("could not ingest invocation output: {error}"))
                .and_then(|value| {
                    builders
                        .complete_pending(arena, borrow_graph, slot, value)
                        .map_err(|error| error.to_string())
                });
            let _ = response.send(result);
        }
        ArenaMutationCommand::Delete { slot, response } => {
            let result = builders
                .delete(arena, borrow_graph, slot)
                .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
    }
}

fn apply_deferred(
    arena: &mut Arena,
    builders: &mut BuilderStore,
    borrow_graph: &mut BorrowGraph,
    invocations: &mut InvocationController,
    productions: &mut ProductionController,
    invocation_host: &mut dyn InvocationHost,
    deferred: std::collections::VecDeque<ArenaMutationCommand>,
) {
    for command in deferred {
        apply_mutation(
            arena,
            builders,
            borrow_graph,
            invocations,
            productions,
            invocation_host,
            command,
        );
    }
}

fn apply_read(
    arena: &Arena,
    builders: &BuilderStore,
    borrow_graph: &BorrowGraph,
    json_encoder: &mut JsonEncoder,
    command: ArenaReadCommand,
) {
    let source = ArenaAddressSource::new(arena);
    apply_read_with_source(
        arena,
        &source,
        builders,
        borrow_graph,
        json_encoder,
        command,
    );
}

fn apply_read_with_source(
    arena: &Arena,
    source: &ArenaAddressSource<'_>,
    builders: &BuilderStore,
    borrow_graph: &BorrowGraph,
    json_encoder: &mut JsonEncoder,
    command: ArenaReadCommand,
) {
    match command {
        ArenaReadCommand::ResolveJson { address, response } => {
            let result = source
                .resolve(&address)
                .map_err(|error| format!("address {address:?} does not resolve: {error}"))
                .and_then(|value| {
                    json_encoder
                        .encode(value.peek())
                        .map_err(|error| format!("could not serialize {address:?}: {error}"))
                });
            let _ = response.send(result);
        }
        ArenaReadCommand::InspectFieldCandidate {
            destination,
            field,
            source,
            response,
        } => {
            let result = super::field_candidate_action::FieldCandidateActions::inspect(
                arena,
                builders,
                borrow_graph,
                destination,
                field,
                source,
            );
            let _ = response.send(result);
        }
        ArenaReadCommand::InspectRoot {
            slot,
            max_relationship_rows,
            response,
        } => {
            let result = super::root_snapshot::RootSnapshot::observe(
                arena,
                builders,
                slot,
                max_relationship_rows,
            )
            .map_err(|error| error.to_string());
            let _ = response.send(result);
        }
        ArenaReadCommand::InspectTab { slot, response } => {
            let _ = response.send(clone_tab(arena, slot));
        }
        ArenaReadCommand::InspectBreadcrumbContext {
            breadcrumbs,
            max_work,
            max_choices,
            response,
        } => {
            let _ = response.send(
                super::breadcrumb_context_snapshot::BreadcrumbContextSnapshot::inspect(
                    source,
                    breadcrumbs,
                    max_work,
                    max_choices,
                ),
            );
        }
        ArenaReadCommand::InspectBreadcrumbValues {
            breadcrumbs,
            field_shape,
            field_name,
            max_work,
            max_choices,
            response,
        } => {
            let _ = response.send(
                super::breadcrumb_context_snapshot::BreadcrumbContextSnapshot::inspect_values(
                    source,
                    breadcrumbs,
                    &field_shape,
                    &field_name,
                    max_work,
                    max_choices,
                ),
            );
        }
    }
}

fn clone_tab(arena: &Arena, slot: super::slot_id::SlotId) -> Result<Tab, String> {
    let value = arena
        .resolve_root(slot)
        .map_err(|error| error.to_string())?;
    if !value.shape().is_shape(Tab::SHAPE) {
        return Err(format!(
            "slot {slot} contains {}, not Tab",
            cloud_terrastodon_registry::describe_shape(value.shape())
        ));
    }
    let value = value
        .try_clone()
        .map_err(|error| format!("could not clone Tab in slot {slot}: {error}"))?;
    value
        .into_box::<Tab>()
        .map_err(|error| format!("could not recover Tab in slot {slot}: {error}"))?
        .downcast::<Tab>()
        .map(|tab| *tab)
        .map_err(|_| format!("slot {slot} did not retain its registered Tab type"))
}

fn reject_inactive_query(command: ArenaQueryCommand) {
    match command {
        ArenaQueryCommand::BeginExport { .. } => unreachable!("begin handled by caller"),
        ArenaQueryCommand::NextJsonBatch { response, .. } => {
            let _ = response.send(Err("no export session is active".to_owned()));
        }
        ArenaQueryCommand::EndExport { response, .. } => {
            let _ = response.send(Err("no export session is active".to_owned()));
        }
    }
}

fn reject_wrong_session(command: ArenaQueryCommand, active: QuerySessionId) {
    match command {
        ArenaQueryCommand::BeginExport { response, .. } => {
            let _ = response.send(Err(format!(
                "export session {} is already active",
                active.get()
            )));
        }
        ArenaQueryCommand::NextJsonBatch {
            session, response, ..
        } => {
            let _ = response.send(Err(format!(
                "export session {} is active, not {}",
                active.get(),
                session.get()
            )));
        }
        ArenaQueryCommand::EndExport {
            session, response, ..
        } => {
            let _ = response.send(Err(format!(
                "export session {} is active, not {}",
                active.get(),
                session.get()
            )));
        }
    }
}

fn reject_inactive_browse(command: BrowseCommand) {
    match command {
        BrowseCommand::Begin { .. } => unreachable!("begin handled by caller"),
        BrowseCommand::SetQuery { response, .. }
        | BrowseCommand::SetCandidateShape { response, .. }
        | BrowseCommand::ClearValueCandidates { response, .. }
        | BrowseCommand::End { response, .. } => {
            let _ = response.send(Err("no browse session is active".to_owned()));
        }
        BrowseCommand::FillCardWindow { response, .. } => {
            let _ = response.send(Err("no browse session is active".to_owned()));
        }
        BrowseCommand::Navigate { response, .. } => {
            let _ = response.send(Err("no browse session is active".to_owned()));
        }
        BrowseCommand::FillValueCandidates { response, .. } => {
            let _ = response.send(Err("no browse session is active".to_owned()));
        }
    }
}

fn reject_wrong_browse_session(command: BrowseCommand, active: BrowseSessionId) {
    match command {
        BrowseCommand::Begin { response, .. } => {
            let _ = response.send(Err(format!(
                "browse session {} is already active",
                active.get()
            )));
        }
        BrowseCommand::SetQuery {
            session, response, ..
        }
        | BrowseCommand::SetCandidateShape {
            session, response, ..
        }
        | BrowseCommand::ClearValueCandidates { session, response }
        | BrowseCommand::End {
            session, response, ..
        } => {
            let _ = response.send(Err(format!(
                "browse session {} is active, not {}",
                active.get(),
                session.get()
            )));
        }
        BrowseCommand::FillCardWindow {
            session, response, ..
        } => {
            let _ = response.send(Err(format!(
                "browse session {} is active, not {}",
                active.get(),
                session.get()
            )));
        }
        BrowseCommand::Navigate {
            session, response, ..
        } => {
            let _ = response.send(Err(format!(
                "browse session {} is active, not {}",
                active.get(),
                session.get()
            )));
        }
        BrowseCommand::FillValueCandidates {
            session, response, ..
        } => {
            let _ = response.send(Err(format!(
                "browse session {} is active, not {}",
                active.get(),
                session.get()
            )));
        }
    }
}

fn reject_browse_during_export(command: BrowseCommand) {
    const MESSAGE: &str = "a coherent export is active; browse work is temporarily unavailable";
    match command {
        BrowseCommand::Begin { response, .. } => {
            let _ = response.send(Err(MESSAGE.to_owned()));
        }
        BrowseCommand::SetQuery { response, .. }
        | BrowseCommand::SetCandidateShape { response, .. }
        | BrowseCommand::ClearValueCandidates { response, .. }
        | BrowseCommand::End { response, .. } => {
            let _ = response.send(Err(MESSAGE.to_owned()));
        }
        BrowseCommand::FillCardWindow { response, .. } => {
            let _ = response.send(Err(MESSAGE.to_owned()));
        }
        BrowseCommand::Navigate { response, .. } => {
            let _ = response.send(Err(MESSAGE.to_owned()));
        }
        BrowseCommand::FillValueCandidates { response, .. } => {
            let _ = response.send(Err(MESSAGE.to_owned()));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::borrow::Cow;
    use std::future::{Future, IntoFuture, pending};
    use std::pin::Pin;

    use cloud_terrastodon_registry::{
        Function, FunctionKind, InvocationFuture, ProductionKind, RegistrationSite, RuntimeValue,
        Thing, functions_from, runtime_from_boxed, runtime_into_boxed,
    };
    use facet::Facet;

    use super::*;
    use crate::object_explorer::arena_query_context::{
        ArenaQueryContext, ArenaQueryContextError, ArenaQuerySession,
    };
    use crate::object_explorer::arena_query_session::{JsonBatch, JsonBatchBudget};
    use crate::object_explorer::arena_slot_state::ArenaSlotState;
    use crate::object_explorer::borrow_graph::BorrowGraph;
    use crate::object_explorer::breadcrumb::{Breadcrumb, ValueFilterOperator};
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::card_address::CardAddress;
    use crate::object_explorer::explorer_command::{
        ExplorerHandleError, FieldBindingPacket, OwnedValuePacket,
    };
    use crate::object_explorer::field_binding::FieldBinding;
    use crate::object_explorer::field_candidate_action::FieldCandidateAction;
    use crate::object_explorer::produce_json_request::ProduceJsonRequest;
    use crate::object_explorer::production_job::{ProductionJobState, ProductionStrategy};
    use crate::object_explorer::selection::CardSelection;
    use crate::object_explorer::tab::Tab;
    use crate::object_explorer::value_address::ValueAddress;
    use crate::object_explorer::value_builder::{BuilderStore, BuilderTransition, ValueBuilder};
    use crate::object_explorer::value_candidate::ValueOwner;
    use crate::object_explorer::value_candidate::scan_value_candidates;
    use crate::object_explorer::value_candidate_window::ValueCandidateWindowBudget;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct TestTab {
        name: String,
        breadcrumbs: Vec<String>,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct EngineBuildPair {
        first: String,
        second: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    enum EngineChoice {
        Empty,
        Named { label: String },
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EngineDefaults {
        #[facet(default)]
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EngineBorrowSource {
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EngineBorrowRequest<'a> {
        source: Cow<'a, EngineBorrowSource>,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EngineBreadcrumbBorrower<'a> {
        breadcrumbs: Cow<'a, Breadcrumbs>,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct EngineMoveTarget {
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EngineInvocationRequest {
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct EngineDefaultProducerRequest {
        marker: u8,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct EngineProducedValue {
        marker: u8,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct EngineProductionDestination {
        value: EngineProducedValue,
    }

    impl IntoFuture for EngineDefaultProducerRequest {
        type Output = eyre::Result<EngineProducedValue>;
        type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(async move {
                Ok(EngineProducedValue {
                    marker: self.marker,
                })
            })
        }
    }

    cloud_terrastodon_registry::register_thing!(EngineDefaultProducerRequest);
    cloud_terrastodon_registry::register_thing!(EngineProducedValue);
    cloud_terrastodon_registry::register_into_future!(
        EngineDefaultProducerRequest => EngineProducedValue
    );

    fn invoke_engine_request(input: Box<dyn Any + Send>) -> InvocationFuture {
        Box::pin(async move {
            let request = input
                .downcast::<EngineInvocationRequest>()
                .map_err(|_| eyre::eyre!("wrong engine request input"))?;
            Ok(Box::new(format!("result: {}", request.value)) as Box<dyn Any + Send>)
        })
    }

    static ENGINE_INVOCATION_THING: Thing = Thing::value(
        EngineInvocationRequest::SHAPE,
        runtime_from_boxed::<EngineInvocationRequest>,
        runtime_into_boxed::<EngineInvocationRequest>,
        RegistrationSite::new(file!(), line!()),
    );

    static ENGINE_INVOCATION_FUNCTION: Function = Function::async_value(
        EngineInvocationRequest::SHAPE,
        String::SHAPE,
        FunctionKind::AsyncInvoke,
        "invoke",
        "engine test",
        &[],
        invoke_engine_request,
        runtime_from_boxed::<String>,
        RegistrationSite::new(file!(), line!()),
    );

    fn engine_default_producer() -> &'static Function {
        functions_from(EngineDefaultProducerRequest::SHAPE)
            .into_iter()
            .find(|function| {
                function.production_kind(EngineProducedValue::SHAPE) == Some(ProductionKind::Exact)
            })
            .expect("engine default producer is registered")
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    async fn append_until_complete(
        session: &ArenaQuerySession,
        document: &mut String,
        first: Option<JsonBatch>,
    ) -> Result<(), ArenaQueryContextError> {
        let mut next = first;
        for _ in 0..100 {
            let batch = match next.take() {
                Some(batch) => batch,
                None => session.next_json_batch(JsonBatchBudget::new(2, 96)).await?,
            };
            assert!(batch.inspected <= 2);
            assert!(batch.fragment.len() <= 96);
            document.push_str(&batch.fragment);
            if batch.complete {
                return Ok(());
            }
        }
        panic!("finite fixture export did not complete within its expected batches")
    }

    #[tokio::test]
    async fn ordinary_registry_invocation_runs_concurrently_and_ingests_through_engine_commands() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();
        let client = async move {
            let input = handle
                .insert_ready(OwnedValuePacket::new(EngineInvocationRequest {
                    value: "linear engine".to_owned(),
                }))
                .await
                .unwrap();
            let start = handle
                .invoke(
                    input,
                    &ENGINE_INVOCATION_THING,
                    &ENGINE_INVOCATION_FUNCTION,
                    super::super::invocation_mode::InvocationMode::Retain,
                )
                .await
                .unwrap();
            let output = start.output();
            let pending = handle.inspect_root(output, 2).await.unwrap();
            assert_eq!(
                pending.lifecycle(),
                &super::super::root_snapshot::RootLifecycleSnapshot::Pending
            );

            let event = loop {
                let mut events = handle.poll_invocations().await.unwrap();
                if let Some(event) = events.pop() {
                    break event;
                }
                tokio::task::yield_now().await;
            };
            assert_eq!(event.output, output);
            assert_eq!(
                event.state,
                super::super::invocation_controller::InvocationEventState::Ready
            );
            let ready = handle.inspect_root(output, 2).await.unwrap();
            assert_eq!(
                ready.lifecycle(),
                &super::super::root_snapshot::RootLifecycleSnapshot::Ready
            );
            drop(handle);
            drop(context);
            (input, output)
        };

        let (engine, (input, output)) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(input).is_some());
        let output = engine
            .arena()
            .ready_value(output)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<String>()
            .unwrap()
            .downcast::<String>()
            .unwrap();
        assert_eq!(output.as_str(), "result: linear engine");
        assert_eq!(engine.pending_invocation_count(), 0);
    }

    #[tokio::test]
    async fn producer_jobs_are_bounded_linear_engine_commands_with_real_roots() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();
        let client = async move {
            let (destination, transition) = handle
                .create_builder(EngineProductionDestination::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            let start = handle
                .start_production(
                    destination,
                    0,
                    engine_default_producer(),
                    ProductionStrategy::Default,
                    1,
                )
                .await
                .unwrap();
            assert_eq!(start.work_spent(), 1);
            assert_eq!(start.active_jobs(), 1);
            let input = start.updates()[0]
                .input()
                .expect("first bounded step creates a visible request root");

            let complete = loop {
                handle.poll_invocations().await.unwrap();
                let batch = handle.advance_productions(1).await.unwrap();
                assert!(batch.work_spent() <= 1);
                if let Some(update) = batch
                    .updates()
                    .iter()
                    .find(|update| update.state().is_terminal())
                {
                    break update.clone();
                }
                tokio::task::yield_now().await;
            };
            assert!(matches!(
                complete.state(),
                ProductionJobState::Complete {
                    destination_transition: BuilderTransition::Ready,
                    ..
                }
            ));
            let ready = handle.inspect_root(destination, 4).await.unwrap();
            assert_eq!(
                ready.lifecycle(),
                &super::super::root_snapshot::RootLifecycleSnapshot::Ready
            );
            drop(handle);
            drop(context);
            (destination, input)
        };

        let (engine, (destination, input)) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.active_production_count(), 0);
        assert!(engine.arena().ready_value(destination).is_some());
        assert!(matches!(
            engine.arena().slot(input).unwrap().state(),
            ArenaSlotState::Consumed
        ));
        assert_eq!(engine.arena().allocated_slot_count(), 4);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn linearly_serviced_export_is_coherent_bounded_and_read_fair() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(TestTab {
                name: "everything".to_owned(),
                breadcrumbs: Vec::new(),
            }))
            .unwrap();
        let stable = arena
            .insert_ready(runtime(String::from("before-export")))
            .unwrap();
        let pending_slot = arena.insert_pending().unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();

        let client = async move {
            let session = context
                .open_export(Breadcrumbs::default())
                .await
                .expect("export opens");
            let first = session
                .next_json_batch(JsonBatchBudget::new(1, 96))
                .await
                .expect("first bounded batch");
            assert!(first.inspected <= 1);

            // Model a network future becoming ready independently. Only the
            // packet's later ingestion may revise Arena.
            let completed_packet =
                tokio::spawn(async { OwnedValuePacket::new(String::from("during-export")) })
                    .await
                    .expect("background future completes");
            let mut receipt = handle
                .submit_set_ready(pending_slot, completed_packet)
                .await
                .expect("completion command enters the linear inbox");

            // This read was enqueued after the mutation through the same
            // handle. Receiving it proves the engine observed and deferred
            // the mutation while continuing to serve read-only work.
            assert_eq!(
                handle
                    .resolve_json(ValueAddress::root(stable))
                    .await
                    .expect("read remains serviceable"),
                "\"before-export\""
            );
            assert_eq!(receipt.try_result().expect("receipt remains valid"), None);

            let mut first_document = String::from("[");
            append_until_complete(&session, &mut first_document, Some(first))
                .await
                .expect("first export completes");
            first_document.push(']');
            assert!(!first_document.contains("during-export"));

            session.complete().await.expect("barrier closes cleanly");
            receipt
                .wait()
                .await
                .expect("deferred completion is ingested after close");

            let second = context
                .open_export(Breadcrumbs::default())
                .await
                .expect("second export opens");
            let mut second_document = String::from("[");
            append_until_complete(&second, &mut second_document, None)
                .await
                .expect("second export completes");
            second_document.push(']');
            second.complete().await.expect("second barrier closes");
            assert!(second_document.contains("during-export"));

            drop(handle);
            drop(context);
            (first_document, second_document)
        };

        let (engine, (first_document, second_document)) = tokio::join!(engine.run(inbox), client);

        assert!(!first_document.contains("during-export"));
        assert!(second_document.contains("during-export"));
        assert_eq!(
            engine
                .arena()
                .ready_value(pending_slot)
                .and_then(|value| value.peek().as_str()),
            Some("during-export")
        );
        assert!(
            engine.json_serialization_count() > 0,
            "explicit reads and export batches must cross the instrumented JSON boundary"
        );
    }

    #[tokio::test]
    async fn export_protocol_bounds_no_match_scans_by_raw_address_work() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);

        let client = async move {
            let session = context
                .open_export(Breadcrumbs::new(vec![
                    Breadcrumb::ShapeFilter {
                        included_shapes: vec![cloud_terrastodon_registry::describe_shape(
                            usize::SHAPE,
                        )],
                    },
                    Breadcrumb::ValueFilter {
                        field_shape: "*".to_owned(),
                        field_name: "missing".to_owned(),
                        operator: ValueFilterOperator::Equals,
                        value: "never".to_owned(),
                    },
                ]))
                .await
                .expect("export opens");

            let first = session
                .next_json_batch(JsonBatchBudget::new(7, 128))
                .await
                .expect("first bounded scan advances");
            let second = session
                .next_json_batch(JsonBatchBudget::new(11, 128))
                .await
                .expect("second bounded scan resumes");

            assert_eq!(first.inspected, 7);
            assert_eq!(second.inspected, 11);
            assert_eq!(first.emitted + second.emitted, 0);
            assert!(!first.complete);
            assert!(!second.complete);

            session
                .cancel()
                .await
                .expect("partial scan cancels cleanly");
            drop(context);
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(
            engine.json_serialization_count(),
            0,
            "unmatched addresses never cross the JSON boundary"
        );
    }

    #[tokio::test]
    async fn browse_card_window_is_bounded_and_rebuilds_after_mutation() {
        let mut arena = Arena::default();
        let large = arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();

        let client = async move {
            let browse = context
                .open_browse(Breadcrumbs::default())
                .await
                .expect("browse opens");
            let first = browse
                .fill_card_window(
                    None,
                    super::super::browse_session::CardWindowBudget::new(
                        16,
                        NonZeroUsize::new(8).unwrap(),
                        3,
                    ),
                )
                .await
                .expect("first bounded card window");
            assert_eq!(first.work_spent(), 9, "eight cards plus one lookahead");
            let first = match first.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected a ready first window, got {state:?}"),
            };
            assert_eq!(first.cards().len(), 8);
            assert!(!first.has_before());
            assert!(first.has_after());
            assert_eq!(first.cards()[0].rows().len(), 4);
            assert!(!first.cards()[0].relationships_complete());
            let anchor = match first.cards()[4].address() {
                CardAddress::Value(address) => address.clone(),
                CardAddress::NewSlot => panic!("observed cards have value addresses"),
            };

            let inserted = handle
                .insert_ready(OwnedValuePacket::new(String::from("after-window")))
                .await
                .expect("mutation is applied after dropping the reflected cursor");

            let rebuilt = browse
                .fill_card_window(
                    Some(anchor.clone()),
                    super::super::browse_session::CardWindowBudget::new(
                        32,
                        NonZeroUsize::new(8).unwrap(),
                        3,
                    ),
                )
                .await
                .expect("browse cursor rebuilds against the new Arena revision");
            assert!(rebuilt.work_spent() <= 32);
            let rebuilt = match rebuilt.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected a rebuilt card window, got {state:?}"),
            };
            assert_eq!(
                rebuilt.cards().first().map(|card| card.address()),
                Some(&CardAddress::Value(anchor))
            );

            browse.close().await.expect("browse closes");
            drop(handle);
            drop(context);
            inserted
        };

        let (engine, inserted) = tokio::join!(engine.run(inbox), client);
        assert_eq!(
            engine
                .arena()
                .ready_value(inserted)
                .and_then(|value| value.peek().as_str()),
            Some("after-window")
        );
        assert_eq!(
            engine.arena().allocated_slot_count(),
            2,
            "a million reflected elements remain one owned root plus the inserted root"
        );
        assert!(engine.arena().ready_value(large).is_some());
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn browse_no_match_scan_spends_exactly_each_frame_budget() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);

        let client = async move {
            let browse = context
                .open_browse(Breadcrumbs::new(vec![
                    Breadcrumb::ShapeFilter {
                        included_shapes: vec![cloud_terrastodon_registry::describe_shape(
                            usize::SHAPE,
                        )],
                    },
                    Breadcrumb::ValueFilter {
                        field_shape: "*".to_owned(),
                        field_name: "missing".to_owned(),
                        operator: ValueFilterOperator::Equals,
                        value: "never".to_owned(),
                    },
                ]))
                .await
                .expect("browse opens");

            for expected in [7, 11] {
                let progress = browse
                    .fill_card_window(
                        None,
                        super::super::browse_session::CardWindowBudget::new(
                            expected,
                            NonZeroUsize::new(8).unwrap(),
                            3,
                        ),
                    )
                    .await
                    .expect("bounded no-match scan advances");
                assert_eq!(progress.work_spent(), expected);
                assert!(matches!(progress.state(), QueryProgressState::Pending));
            }

            browse.close().await.expect("partial browse closes");
            drop(context);
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn browse_query_replacement_changes_results_without_replacing_session() {
        let mut arena = Arena::default();
        let text = arena
            .insert_ready(runtime(String::from("selected text")))
            .unwrap();
        let number = arena.insert_ready(runtime(42_usize)).unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);

        let client = async move {
            let browse = context
                .open_browse(Breadcrumbs::default())
                .await
                .expect("browse opens");
            let session = browse.id();
            browse
                .set_query(Breadcrumbs::new(vec![Breadcrumb::ShapeFilter {
                    included_shapes: vec![cloud_terrastodon_registry::describe_shape(usize::SHAPE)],
                }]))
                .await
                .expect("query replacement accepted");
            assert_eq!(browse.id(), session);

            let progress = browse
                .fill_card_window(
                    None,
                    super::super::browse_session::CardWindowBudget::new(
                        8,
                        NonZeroUsize::new(1).unwrap(),
                        0,
                    ),
                )
                .await
                .expect("replacement query evaluates");
            let window = match progress.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected replacement result, got {state:?}"),
            };
            assert_eq!(
                window.cards()[0].address(),
                &CardAddress::Value(ValueAddress::root(number))
            );

            browse.close().await.expect("browse closes");
            drop(context);
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(text).is_some());
        assert!(engine.arena().ready_value(number).is_some());
    }

    #[tokio::test]
    async fn browse_candidate_windows_report_projected_field_owners_generically() {
        let mut arena = Arena::default();
        for name in ["first", "second", "third"] {
            arena
                .insert_ready(runtime(Tab::new(name, Breadcrumbs::default())))
                .unwrap();
        }
        let allocated_roots = arena.allocated_slot_count();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);

        let client = async move {
            let browse = context
                .open_browse(Breadcrumbs::default())
                .await
                .expect("browse opens");
            browse
                .set_candidate_shape(Breadcrumbs::SHAPE)
                .await
                .expect("candidate shape is ordinary reflected metadata");
            let progress = browse
                .fill_value_candidates(
                    None,
                    ValueCandidateWindowBudget::new(32, NonZeroUsize::new(2).unwrap()),
                )
                .await
                .expect("bounded candidate window resolves");
            assert!(progress.work_spent() <= 32);
            let window = match progress.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected candidate window, got {state:?}"),
            };
            assert_eq!(window.candidates().len(), 2);
            assert!(!window.has_before());
            assert!(window.has_after());
            for candidate in window.candidates() {
                assert_eq!(candidate.shape(), "Breadcrumbs");
                assert!(matches!(
                    candidate.owner(),
                    ValueOwner::ReflectedField {
                        owner_shape,
                        field,
                        ..
                    } if owner_shape == "Tab" && field == "breadcrumbs"
                ));
                assert!(
                    candidate
                        .display_label()
                        .contains(" — field breadcrumbs of slot")
                );
                assert!(candidate.display_label().contains("(Tab)"));
            }

            browse
                .clear_value_candidates()
                .await
                .expect("closing the picker releases its cursor");
            let error = browse
                .fill_value_candidates(
                    None,
                    ValueCandidateWindowBudget::new(4, NonZeroUsize::new(1).unwrap()),
                )
                .await
                .expect_err("a closed picker scan has no hidden fallback");
            assert!(matches!(
                error,
                ArenaQueryContextError::Rejected(message)
                    if message.contains("no value-candidate shape")
            ));

            browse.close().await.expect("browse closes");
            drop(context);
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.arena().allocated_slot_count(), allocated_roots);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn million_value_candidate_window_is_address_and_work_bounded() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);

        let client = async move {
            let browse = context
                .open_browse(Breadcrumbs::default())
                .await
                .expect("browse opens");
            browse
                .set_candidate_shape(usize::SHAPE)
                .await
                .expect("candidate shape accepted");
            let progress = browse
                .fill_value_candidates(
                    None,
                    ValueCandidateWindowBudget::new(16, NonZeroUsize::new(8).unwrap()),
                )
                .await
                .expect("first candidate window resolves");
            assert_eq!(
                progress.work_spent(),
                10,
                "one Vec root, eight candidates, and one candidate lookahead"
            );
            let window = match progress.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected candidate window, got {state:?}"),
            };
            assert_eq!(window.candidates().len(), 8);
            assert!(window.has_after());
            assert!(window.candidates().iter().all(|candidate| matches!(
                candidate.owner(),
                ValueOwner::SequenceElement { owner_shape, .. }
                    if owner_shape == "List<usize>"
            )));

            browse.close().await.expect("browse closes");
            drop(context);
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.arena().allocated_slot_count(), 1);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn cow_picker_consequences_offer_borrow_move_and_clone_without_mutating() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();

        let client = async move {
            let source = handle
                .insert_ready(OwnedValuePacket::new(EngineBorrowSource {
                    value: "organization".to_owned(),
                }))
                .await
                .unwrap();
            let (request, transition) = handle
                .create_builder(<EngineBorrowRequest<'static>>::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);

            let browse = context
                .open_browse(Breadcrumbs::default())
                .await
                .expect("browse session opens");
            browse
                .set_candidate_shape(<Cow<'static, EngineBorrowSource>>::SHAPE)
                .await
                .expect("Cow field shape starts generic candidate discovery");
            let candidates = browse
                .fill_value_candidates(
                    None,
                    ValueCandidateWindowBudget::new(8, NonZeroUsize::new(4).unwrap()),
                )
                .await
                .expect("Cow pointee candidates resolve");
            let candidates = match candidates.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected Cow candidate window, got {state:?}"),
            };
            assert!(
                candidates
                    .candidates()
                    .iter()
                    .any(|candidate| candidate.address() == &ValueAddress::root(source)),
                "a compatible pointee must be discoverable from the Cow field shape"
            );

            let options = handle
                .inspect_field_candidate(request, 0, ValueAddress::root(source))
                .await
                .expect("Cow pointee is a valid generic picker candidate");
            assert_eq!(
                options
                    .consequences()
                    .iter()
                    .map(|consequence| consequence.action())
                    .collect::<Vec<_>>(),
                [
                    FieldCandidateAction::Borrow,
                    FieldCandidateAction::Move,
                    FieldCandidateAction::Clone,
                ]
            );
            assert!(
                options
                    .consequences()
                    .iter()
                    .all(|consequence| consequence.description().contains("slot 0"))
            );

            browse.close().await.unwrap();

            drop(handle);
            drop(context);
            (source, request)
        };

        let (engine, (source, request)) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(source).is_some());
        assert!(engine.builders().builder(request).is_some());
        assert_eq!(engine.borrow_graph().edge_count(), 0);
    }

    #[tokio::test]
    async fn projected_breadcrumb_picker_consequence_names_its_tab_owner_and_only_clones() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();

        let client = async move {
            let tab = handle
                .insert_ready(OwnedValuePacket::new(Tab::new(
                    "admins",
                    Breadcrumbs::default(),
                )))
                .await
                .unwrap();
            let (request, transition) = handle
                .create_builder(ProduceJsonRequest::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            let breadcrumbs =
                ValueAddress::root(tab).child(ValuePathSegment::Field("breadcrumbs".to_owned()));

            let options = handle
                .inspect_field_candidate(request, 0, breadcrumbs.clone())
                .await
                .expect("projected Breadcrumbs can populate the ordinary field");
            assert_eq!(options.candidate().address(), &breadcrumbs);
            assert!(
                options
                    .candidate()
                    .display_label()
                    .contains("field breadcrumbs of slot 0 (Tab)")
            );
            assert_eq!(options.consequences().len(), 1);
            assert_eq!(
                options.consequences()[0].action(),
                FieldCandidateAction::Clone
            );
            assert!(
                options.consequences()[0]
                    .description()
                    .contains("containing field remain unchanged")
            );

            drop(handle);
            drop(context);
            (tab, request)
        };

        let (engine, (tab, request)) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(tab).is_some());
        assert!(engine.builders().builder(request).is_some());
        assert_eq!(engine.arena().allocated_slot_count(), 2);
    }

    #[tokio::test]
    async fn dropping_browse_session_releases_engine_without_async_drop() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(2);
        let handle = context.engine_handle();

        let client = async move {
            let browse = context
                .open_browse(Breadcrumbs::default())
                .await
                .expect("browse opens");
            drop(browse);

            let inserted = handle
                .insert_ready(OwnedValuePacket::new(String::from("after-browse-drop")))
                .await
                .expect("mutation resumes after synchronous lease cancellation");
            drop(handle);
            drop(context);
            inserted
        };

        let (engine, inserted) = tokio::join!(engine.run(inbox), client);
        assert_eq!(
            engine
                .arena()
                .ready_value(inserted)
                .and_then(|value| value.peek().as_str()),
            Some("after-browse-drop")
        );
    }

    #[tokio::test]
    async fn aborting_export_future_releases_barrier_without_async_drop() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(TestTab {
                name: "abortable".to_owned(),
                breadcrumbs: Vec::new(),
            }))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(2);
        let handle = context.engine_handle();

        let client = async move {
            let task_context = context.clone();
            let (opened, opened_receiver) = oneshot::channel();
            let producer = tokio::spawn(async move {
                let _session = task_context
                    .open_export(Breadcrumbs::default())
                    .await
                    .expect("export opens");
                opened.send(()).expect("test awaits open signal");
                pending::<()>().await;
            });

            opened_receiver.await.expect("producer owns a session");
            producer.abort();
            assert!(
                producer
                    .await
                    .expect_err("producer is aborted")
                    .is_cancelled()
            );

            // The cancellation lease is independent of queue capacity, so the
            // next mutation completes instead of waiting behind a leaked
            // export barrier.
            let inserted = handle
                .insert_ready(OwnedValuePacket::new(String::from("after-abort")))
                .await
                .expect("mutation resumes after abort cleanup");
            assert_eq!(
                handle
                    .resolve_json(ValueAddress::root(inserted))
                    .await
                    .expect("inserted value resolves"),
                "\"after-abort\""
            );

            drop(handle);
            drop(context);
            inserted
        };

        let (engine, inserted) = tokio::join!(engine.run(inbox), client);
        assert_eq!(
            engine
                .arena()
                .ready_value(inserted)
                .and_then(|value| value.peek().as_str()),
            Some("after-abort")
        );
    }

    #[tokio::test]
    async fn ordinary_explorer_actions_do_not_serialize_json() {
        let mut arena = Arena::default();
        let tab = arena
            .insert_ready(runtime(Tab::new("source", Breadcrumbs::default())))
            .unwrap();
        let selected_slot = arena
            .insert_ready(runtime(String::from("selected")))
            .unwrap();

        let breadcrumbs_address = {
            let source = ArenaAddressSource::new(&arena);
            let mut selection =
                CardSelection::new(CardAddress::Value(ValueAddress::root(selected_slot)));
            selection.reconcile(&source);
            assert_eq!(
                selection.selected(),
                &CardAddress::Value(ValueAddress::root(selected_slot))
            );

            let mut cursor = PreorderCursor::new(&source);
            let batch = scan_value_candidates(
                &mut cursor,
                &source,
                Breadcrumbs::SHAPE,
                WorkBudget::new(128),
            );
            assert!(batch.complete);
            batch
                .candidates
                .into_iter()
                .find(|candidate| {
                    candidate.address()
                        == &ValueAddress::root(tab)
                            .child(ValuePathSegment::Field("breadcrumbs".to_owned()))
                })
                .expect("the generic picker sees Tab.breadcrumbs")
                .address()
                .clone()
        };

        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        assert_eq!(
            builders
                .insert_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    ValueBuilder::new(ProduceJsonRequest::SHAPE),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::CloneFrom(breadcrumbs_address),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    1,
                    FieldBinding::InlineOwned(runtime(String::from("admins.json"))),
                )
                .unwrap(),
            BuilderTransition::Ready
        );

        let engine = ExplorerEngine::new(arena);
        assert_eq!(engine.json_serialization_count(), 0);
        let (context, inbox) = ArenaQueryContext::channel(2);
        let handle = context.engine_handle();
        let client = async move {
            let inserted = handle
                .insert_ready(OwnedValuePacket::new(String::from("background result")))
                .await
                .expect("ordinary producer completion is ingested");
            drop(handle);
            drop(context);
            inserted
        };

        let (engine, inserted) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(request).is_some());
        assert!(engine.arena().ready_value(inserted).is_some());
        assert_eq!(
            engine.json_serialization_count(),
            0,
            "navigation, candidate discovery, field assignment, local finalization, and ordinary ingestion must remain reflection-native"
        );
    }

    #[tokio::test]
    async fn engine_builder_commands_finalize_only_the_addressed_builder_before_reply() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();
        let client = async move {
            let pair = handle.reserve_builder().await.unwrap();
            assert!(matches!(
                handle
                    .set_builder_field(
                        pair,
                        0,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(
                            String::from("too early")
                        )),
                    )
                    .await,
                Err(ExplorerHandleError::Rejected(message))
                    if message.contains("has no selected shape")
            ));
            let transition = handle
                .set_builder_shape(pair, EngineBuildPair::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle
                    .set_builder_field(
                        pair,
                        0,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(String::from(
                            "first"
                        ))),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Building
            );
            assert_eq!(
                handle
                    .set_builder_field(pair, 1, FieldBindingPacket::PendingProducer,)
                    .await
                    .unwrap(),
                BuilderTransition::Building
            );
            assert_eq!(
                handle.unset_builder_field(pair, 0).await.unwrap(),
                BuilderTransition::Building
            );
            assert!(matches!(
                handle
                    .complete_builder_field(
                        pair,
                        0,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(
                            String::from("not pending")
                        )),
                    )
                    .await,
                Err(ExplorerHandleError::Rejected(message))
                    if message.contains("is not waiting for a producer")
            ));
            assert!(matches!(
                handle
                    .complete_builder_field(
                        pair,
                        1,
                        FieldBindingPacket::PendingProducer,
                    )
                    .await,
                Err(ExplorerHandleError::Rejected(message))
                    if message.contains("cannot remain PendingProducer")
            ));
            assert_eq!(
                handle
                    .complete_builder_field(
                        pair,
                        1,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(String::from(
                            "second"
                        ))),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Building,
                "producer completion checks this builder but another required field is unset"
            );
            assert_eq!(
                handle
                    .set_builder_field(
                        pair,
                        0,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(String::from(
                            "first again"
                        ))),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            let (scalar, transition) = handle.create_builder(String::SHAPE).await.unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle
                    .set_builder_scalar(scalar, OwnedValuePacket::new(String::from("scalar")),)
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            let (choice, transition) = handle.create_builder(EngineChoice::SHAPE).await.unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle.select_builder_variant(choice, 1).await.unwrap(),
                BuilderTransition::Building
            );
            assert_eq!(
                handle
                    .set_builder_field(
                        choice,
                        0,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(String::from(
                            "chosen"
                        ))),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            let defaults = handle.reserve_builder().await.unwrap();
            let transition = handle
                .set_builder_shape(defaults, EngineDefaults::SHAPE)
                .await
                .unwrap();
            assert_eq!(
                transition,
                BuilderTransition::Ready,
                "a builder that is complete from reflected defaults never escapes as Building"
            );

            let abandoned = handle.reserve_builder().await.unwrap();
            let transition = handle
                .set_builder_shape(abandoned, EngineBuildPair::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            handle.delete(abandoned).await.unwrap();

            drop(handle);
            drop(context);
            (pair, scalar, choice, defaults, abandoned)
        };

        let (engine, (pair, scalar, choice, defaults, abandoned)) =
            tokio::join!(engine.run(inbox), client);
        for slot in [pair, scalar, choice, defaults] {
            assert!(engine.arena().ready_value(slot).is_some());
            assert!(engine.builders().builder(slot).is_none());
        }
        assert_eq!(
            engine
                .arena()
                .ready_value(pair)
                .unwrap()
                .try_clone()
                .unwrap()
                .into_box::<EngineBuildPair>()
                .unwrap()
                .downcast::<EngineBuildPair>()
                .unwrap()
                .as_ref(),
            &EngineBuildPair {
                first: "first again".to_owned(),
                second: "second".to_owned(),
            }
        );
        assert!(engine.builders().builder(abandoned).is_none());
        assert!(matches!(
            engine.arena().slot(abandoned).map(|slot| slot.state()),
            Some(ArenaSlotState::Tombstone {
                previous: "Building"
            })
        ));
    }

    #[tokio::test]
    async fn engine_reply_observes_source_ready_before_following_cow_borrow() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(4);
        let handle = context.engine_handle();
        let client = async move {
            let (source, _) = handle
                .create_builder(EngineBorrowSource::SHAPE)
                .await
                .unwrap();
            assert_eq!(
                handle
                    .set_builder_field(
                        source,
                        0,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(String::from(
                            "resolved"
                        ))),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            let (borrower, _) = handle
                .create_builder(<EngineBorrowRequest<'static>>::SHAPE)
                .await
                .unwrap();
            assert_eq!(
                handle
                    .set_builder_field(
                        borrower,
                        0,
                        FieldBindingPacket::BorrowFrom(ValueAddress::root(source)),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            drop(handle);
            drop(context);
            (source, borrower)
        };

        let (engine, (source, borrower)) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(source).is_some());
        assert!(engine.arena().ready_value(borrower).is_some());
        assert_eq!(engine.borrow_graph().edge_count(), 1);
        assert_eq!(engine.builders().leases(borrower).len(), 1);
    }

    #[tokio::test]
    async fn engine_move_binding_consumes_exactly_the_selected_owned_root() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(String::from("move through engine")))
            .unwrap();
        let engine = ExplorerEngine::new(arena);
        let (context, inbox) = ArenaQueryContext::channel(4);
        let handle = context.engine_handle();
        let client = async move {
            let nested = ValueAddress::root(source).child(ValuePathSegment::Index(0));
            assert!(
                FieldBindingPacket::move_from_address(nested).is_err(),
                "a reflected projection cannot be represented as an owned move"
            );

            let (target, transition) = handle
                .create_builder(EngineMoveTarget::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle
                    .set_builder_field(
                        target,
                        0,
                        FieldBindingPacket::move_from_address(ValueAddress::root(source),).unwrap(),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            drop(handle);
            drop(context);
            target
        };

        let (engine, target) = tokio::join!(engine.run(inbox), client);
        assert!(matches!(
            engine.arena().slot(source).map(|slot| slot.state()),
            Some(ArenaSlotState::Consumed)
        ));
        let value = engine
            .arena()
            .ready_value(target)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<EngineMoveTarget>()
            .unwrap()
            .downcast::<EngineMoveTarget>()
            .unwrap();
        assert_eq!(
            value.as_ref(),
            &EngineMoveTarget {
                value: "move through engine".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn engine_rejects_source_deletion_until_ready_borrower_is_deleted() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(4);
        let handle = context.engine_handle();
        let client = async move {
            let source = handle
                .insert_ready(OwnedValuePacket::new(EngineBorrowSource {
                    value: "engine protected".to_owned(),
                }))
                .await
                .unwrap();
            let (borrower, transition) = handle
                .create_builder(<EngineBorrowRequest<'static>>::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle
                    .set_builder_field(
                        borrower,
                        0,
                        FieldBindingPacket::BorrowFrom(ValueAddress::root(source)),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            assert!(matches!(
                handle.delete(source).await,
                Err(ExplorerHandleError::Rejected(message))
                    if message.contains("protected by 1 borrow lease")
            ));
            handle.delete(borrower).await.unwrap();
            handle.delete(source).await.unwrap();

            drop(handle);
            drop(context);
            (source, borrower)
        };

        let (engine, (source, borrower)) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.borrow_graph().edge_count(), 0);
        for slot in [source, borrower] {
            assert!(matches!(
                engine.arena().slot(slot).map(|slot| slot.state()),
                Some(ArenaSlotState::Tombstone { previous: "Ready" })
            ));
        }
    }

    #[tokio::test]
    async fn tab_deletion_uses_ordinary_descendant_borrows_while_produce_json_does_not_borrow() {
        let engine = ExplorerEngine::new(Arena::default());
        let (context, inbox) = ArenaQueryContext::channel(8);
        let handle = context.engine_handle();
        let client = async move {
            let tab = handle
                .insert_ready(OwnedValuePacket::new(Tab::new(
                    "admins",
                    Breadcrumbs::default(),
                )))
                .await
                .unwrap();
            let breadcrumbs =
                ValueAddress::root(tab).child(ValuePathSegment::Field("breadcrumbs".to_owned()));

            let (export, transition) = handle
                .create_builder(ProduceJsonRequest::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle
                    .set_builder_field(
                        export,
                        0,
                        FieldBindingPacket::CloneFrom(breadcrumbs.clone()),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Building
            );
            assert_eq!(
                handle
                    .set_builder_field(
                        export,
                        1,
                        FieldBindingPacket::InlineOwned(OwnedValuePacket::new(String::from(
                            "admins.json",
                        ))),
                    )
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            let (borrower, transition) = handle
                .create_builder(<EngineBreadcrumbBorrower<'static>>::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            assert_eq!(
                handle
                    .set_builder_field(borrower, 0, FieldBindingPacket::BorrowFrom(breadcrumbs),)
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );

            assert!(matches!(
                handle.delete(tab).await,
                Err(ExplorerHandleError::Rejected(message))
                    if message.contains("protected by 1 borrow lease")
            ));
            assert!(matches!(
                handle
                    .update_tab(
                        tab,
                        crate::object_explorer::tab_update::TabUpdate::Rename(
                            "blocked".to_owned(),
                        ),
                    )
                    .await,
                Err(ExplorerHandleError::Rejected(message))
                    if message.contains("protected by 1 borrow lease")
            ));
            handle.delete(borrower).await.unwrap();
            let updated = handle
                .update_tab(
                    tab,
                    crate::object_explorer::tab_update::TabUpdate::Rename("renamed".to_owned()),
                )
                .await
                .expect("ordinary root replacement resumes when its descendant borrow ends");
            assert_eq!(updated.name(), "renamed");
            assert_eq!(
                handle.inspect_tab(tab).await.unwrap().name(),
                "renamed",
                "updating Tab data preserves its arena SlotId"
            );
            handle
                .delete(tab)
                .await
                .expect("the owned Breadcrumbs clone in ProduceJson is not a Tab borrow");
            handle.delete(export).await.unwrap();

            drop(handle);
            drop(context);
            (tab, borrower, export)
        };

        let (engine, slots) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.borrow_graph().edge_count(), 0);
        for slot in [slots.0, slots.1, slots.2] {
            assert!(matches!(
                engine.arena().slot(slot).map(|slot| slot.state()),
                Some(ArenaSlotState::Tombstone { previous: "Ready" })
            ));
        }
    }
}
