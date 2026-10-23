use crate::discovery_plan_text::edge_label_size;

/// A horizontal crossing of one inter-layer gap. The occupied interval includes
/// its label so assigning separate tracks also separates text rectangles.
pub(crate) struct DiscoveryPlanLayoutSegment {
    pub start_x: f64,
    pub end_x: f64,
    pub destination: usize,
    pub label_x: f64,
    pub label_width: f64,
    pub label_height: f64,
    pub y: f64,
}

impl DiscoveryPlanLayoutSegment {
    pub fn new(start_x: f64, end_x: f64, destination: usize, label: &str) -> Self {
        let (label_width, label_height) = edge_label_size(label);
        let label_x = if (end_x - start_x).abs() >= label_width + 24.0 {
            (start_x + end_x - label_width) / 2.0
        } else {
            start_x.max(end_x) + 12.0
        };
        Self {
            start_x,
            end_x,
            destination,
            label_x,
            label_width,
            label_height,
            y: 0.0,
        }
    }

    /// Keep text out of every vertical routing column in this gap. This includes
    /// dummy runs belonging to other layers, which otherwise can cut through a
    /// long edge's label even when all labels have separate horizontal tracks.
    pub fn place_label_between(&mut self, columns: &[f64]) {
        if self.label_width == 0.0 {
            return;
        }
        let preferred = self.label_x;
        let mut best = f64::INFINITY;
        let mut left = f64::NEG_INFINITY;
        for right in columns.iter().copied().chain([f64::INFINITY]) {
            let minimum = left + 12.0;
            let maximum = right - 12.0 - self.label_width;
            if minimum <= maximum {
                let candidate = preferred.max(minimum).min(maximum);
                if (candidate - preferred).abs() <= (best - preferred).abs() {
                    best = candidate;
                }
            }
            left = right;
        }
        self.label_x = best;
    }

    /// Shared destinations can merge arrow runs, but never their distinct labels.
    pub fn can_share_track(&self, other: &Self) -> bool {
        let labels_overlap = self.label_width > 0.0
            && other.label_width > 0.0
            && self.label_x < other.label_x + other.label_width + 12.0
            && other.label_x < self.label_x + self.label_width + 12.0;
        if labels_overlap {
            return false;
        }
        if self.destination == other.destination && self.end_x == other.end_x {
            return true;
        }
        let (left, right) = self.interval();
        let (other_left, other_right) = other.interval();
        right + 12.0 < other_left || other_right + 12.0 < left
    }

    fn interval(&self) -> (f64, f64) {
        let left = self.start_x.min(self.end_x);
        let right = self.start_x.max(self.end_x);
        if self.label_width > 0.0 {
            (
                left.min(self.label_x),
                right.max(self.label_x + self.label_width),
            )
        } else {
            (left, right)
        }
    }
}

/// Allocate parallel tracks without permuting graph vertices. Rightward tracks
/// are stacked above leftward tracks, following iongraph's outside-in ordering.
/// A fixed minimum margin leaves room for the node bindings and arrowheads.
pub(crate) fn assign_tracks(segments: &mut [DiscoveryPlanLayoutSegment]) -> f64 {
    if segments.is_empty() {
        return 0.0;
    }
    let mut order = (0..segments.len()).collect::<Vec<_>>();
    order.sort_by(|&a, &b| {
        segments[a]
            .start_x
            .total_cmp(&segments[b].start_x)
            .then_with(|| segments[a].end_x.total_cmp(&segments[b].end_x))
            .then_with(|| segments[a].destination.cmp(&segments[b].destination))
            .then_with(|| a.cmp(&b))
    });
    let mut rightward = Vec::<Vec<usize>>::new();
    let mut leftward = Vec::<Vec<usize>>::new();
    for index in order {
        let tracks = if segments[index].end_x >= segments[index].start_x {
            &mut rightward
        } else {
            &mut leftward
        };
        let mut available = None;
        for (position, track) in tracks.iter().enumerate().rev() {
            if track
                .iter()
                .all(|&other| segments[index].can_share_track(&segments[other]))
            {
                available = Some(position);
            } else {
                // Do not cross a blocked outer track to reuse an inner one.
                // This preserves the article's outside-in crossing order.
                break;
            }
        }
        if let Some(position) = available {
            tracks[position].push(index);
        } else {
            tracks.push(vec![index]);
        }
    }
    let label_height = segments
        .iter()
        .map(|segment| segment.label_height)
        .fold(0.0, f64::max);
    let track_height = label_height + 28.0;
    let track_count = rightward.len() + leftward.len();
    for (track_index, track) in rightward.iter().rev().chain(&leftward).enumerate() {
        for &index in track {
            segments[index].y = 24.0 + label_height + track_index as f64 * track_height;
        }
    }
    48.0 + track_count as f64 * track_height
}
