use tokio::sync::oneshot;

use super::arena_query_session::{JsonBatch, QuerySessionEnd, QuerySessionId};
use super::breadcrumbs::Breadcrumbs;

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
    EndExport {
        session: QuerySessionId,
        end: QuerySessionEnd,
        response: oneshot::Sender<CommandResponse<()>>,
    },
}
