use super::arena_query_session::JsonBatch;
use super::arena_query_session::QuerySessionEnd;
use super::arena_query_session::QuerySessionId;
use super::breadcrumbs::Breadcrumbs;
use super::diagram_batch::DiagramBatch;
use super::shape_batch::ShapeBatch;
use tokio::sync::oneshot;

pub(crate) type CommandResponse<T> = Result<T, String>;

/// Value-free command protocol into the single-owner ExplorerEngine.
///
/// RuntimeValue and Facet Peek deliberately cannot appear in these messages.
pub(crate) enum ArenaQueryCommand {
    BeginExport {
        breadcrumbs: Breadcrumbs,
        /// Closing this receiver is the cancellation lease. It lets an
        /// aborted producer release the engine-side barrier without needing
        /// to run async cleanup from Drop or enqueue another bounded command.
        cancelled: oneshot::Receiver<()>,
        response: oneshot::Sender<CommandResponse<QuerySessionId>>,
    },
    NextJsonBatch {
        session: QuerySessionId,
        max_work: usize,
        max_bytes: usize,
        response: oneshot::Sender<CommandResponse<JsonBatch>>,
    },
    NextDiagramBatch {
        session: QuerySessionId,
        max_work: usize,
        max_objects: usize,
        max_rows: usize,
        response: oneshot::Sender<CommandResponse<DiagramBatch>>,
    },
    NextShapeBatch {
        session: QuerySessionId,
        max_work: usize,
        response: oneshot::Sender<CommandResponse<ShapeBatch>>,
    },
    EndExport {
        session: QuerySessionId,
        end: QuerySessionEnd,
        response: oneshot::Sender<CommandResponse<()>>,
    },
}
