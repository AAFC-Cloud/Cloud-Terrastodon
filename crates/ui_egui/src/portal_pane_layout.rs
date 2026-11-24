use crate::portal_pane_id::PortalPaneId;
use crate::portal_split::PortalSplit;
use crate::portal_split_handle::PortalSplitHandle;
use crate::portal_split_side::PortalSplitSide;
use eframe::egui::Rect;

/// A workspace is a pane or a pair of workspaces separated by a divider.
#[derive(Clone)]
pub(crate) enum PortalPaneLayout {
    Pane(PortalPaneId),
    Split {
        /// The pane created by this split also identifies its divider.
        marker: PortalPaneId,
        divider: PortalSplit,
        children: Box<[Self; 2]>,
    },
}

impl PortalPaneLayout {
    pub fn split_handles(&self, bounds: Rect, handles: &mut Vec<PortalSplitHandle>) {
        if let Self::Split {
            marker,
            divider,
            children,
        } = self
            && bounds.is_positive()
        {
            let (first, rect, second) = divider.layout(bounds);
            handles.push(PortalSplitHandle {
                marker: *marker,
                bounds,
                rect,
                horizontal: divider.horizontal(),
            });
            children[0].split_handles(first, handles);
            children[1].split_handles(second, handles);
        }
    }

    pub fn split(
        &mut self,
        destination: PortalPaneId,
        new: PortalPaneId,
        side: PortalSplitSide,
    ) -> bool {
        match self {
            Self::Pane(id) if *id == destination => {
                let existing = Self::Pane(destination);
                let created = Self::Pane(new);
                let children = match side {
                    PortalSplitSide::Left | PortalSplitSide::Top => [created, existing],
                    PortalSplitSide::Right | PortalSplitSide::Bottom => [existing, created],
                };
                *self = Self::Split {
                    marker: new,
                    divider: PortalSplit::new(side),
                    children: Box::new(children),
                };
                true
            }
            Self::Pane(_) => false,
            Self::Split { children, .. } => children
                .iter_mut()
                .any(|child| child.split(destination, new, side)),
        }
    }

    pub fn divider_mut(&mut self, id: PortalPaneId) -> Option<&mut PortalSplit> {
        match self {
            Self::Pane(_) => None,
            Self::Split {
                marker,
                divider,
                children,
            } => {
                if *marker == id {
                    Some(divider)
                } else {
                    children.iter_mut().find_map(|child| child.divider_mut(id))
                }
            }
        }
    }

    pub fn retain(self, contains: impl Fn(PortalPaneId) -> bool + Copy) -> Option<Self> {
        match self {
            Self::Pane(id) => contains(id).then_some(Self::Pane(id)),
            Self::Split {
                marker,
                divider,
                children,
            } => {
                let [first, second] = *children;
                match (first.retain(contains), second.retain(contains)) {
                    (Some(first), Some(second)) => Some(Self::Split {
                        marker,
                        divider,
                        children: Box::new([first, second]),
                    }),
                    (first, second) => first.or(second),
                }
            }
        }
    }
}
