use facet::Shape;
use tokio::sync::oneshot;

use super::browse_session::BrowseSessionId;
use super::card_navigation::CardNavigation;
use super::card_window::CardWindow;
use super::query_progress::QueryProgress;
use super::value_address::ValueAddress;
use super::value_candidate_window::ValueCandidateWindow;
use super::{arena_query_command::CommandResponse, breadcrumbs::Breadcrumbs};

pub(crate) enum BrowseCommand {
    Begin {
        breadcrumbs: Breadcrumbs,
        cancelled: oneshot::Receiver<()>,
        response: oneshot::Sender<CommandResponse<BrowseSessionId>>,
    },
    SetQuery {
        session: BrowseSessionId,
        breadcrumbs: Breadcrumbs,
        response: oneshot::Sender<CommandResponse<()>>,
    },
    FillCardWindow {
        session: BrowseSessionId,
        anchor: Option<ValueAddress>,
        max_work: usize,
        max_cards: usize,
        max_relationship_rows: usize,
        response: oneshot::Sender<CommandResponse<QueryProgress<CardWindow>>>,
    },
    Navigate {
        session: BrowseSessionId,
        from: ValueAddress,
        direction: CardNavigation,
        max_work: usize,
        response: oneshot::Sender<CommandResponse<QueryProgress<ValueAddress>>>,
    },
    SetCandidateShape {
        session: BrowseSessionId,
        target_shape: &'static Shape,
        response: oneshot::Sender<CommandResponse<()>>,
    },
    FillValueCandidates {
        session: BrowseSessionId,
        anchor: Option<ValueAddress>,
        max_work: usize,
        max_candidates: usize,
        response: oneshot::Sender<CommandResponse<QueryProgress<ValueCandidateWindow>>>,
    },
    ClearValueCandidates {
        session: BrowseSessionId,
        response: oneshot::Sender<CommandResponse<()>>,
    },
    End {
        session: BrowseSessionId,
        response: oneshot::Sender<CommandResponse<()>>,
    },
}
