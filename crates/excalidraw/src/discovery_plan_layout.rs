use crate::discovery_plan_layout_edge::DiscoveryPlanLayoutEdge;
use crate::discovery_plan_layout_graph::layers;
use crate::discovery_plan_layout_node::DiscoveryPlanLayoutNode;
use crate::discovery_plan_layout_segment::DiscoveryPlanLayoutSegment;
use crate::discovery_plan_layout_segment::assign_tracks;
use cloud_terrastodon_registry::DiscoveryPlan;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const NODE_SPACING: f64 = 112.0;
const LANE_SPACING: f64 = 32.0;
const LANE_MARGIN: f64 = 80.0;
const BINDING_GAP: f64 = 8.0;
const ALIGNMENT_PASSES: usize = 4;

/// Geometry for a discovery diagram, adapted from Ben Visness's
/// [iongraph layout](https://spidermonkey.dev/blog/2025/10/28/iongraph-web.html).
///
/// Stable ordering, longest-path layers, shared destination dummy runs, bounded
/// alignment, and horizontal tracks replace a global crossing optimizer. Request
/// dependencies do not carry source-code loop nesting, so feedback edges are
/// found by deterministic DFS and routed on the right without inventing loop
/// semantics. The leftward dummy runs have one fixed X coordinate across every
/// crossed layer: this guarantees clear vertical corridors through arbitrary
/// dependency graphs, including blocked nodes whose discovery depth is zero.
#[derive(Debug, PartialEq)]
pub(crate) struct DiscoveryPlanLayout {
    pub nodes: BTreeMap<usize, DiscoveryPlanLayoutNode>,
    pub edges: BTreeMap<usize, DiscoveryPlanLayoutEdge>,
    pub width: f64,
    pub height: f64,
}

impl DiscoveryPlanLayout {
    pub fn new(plan: &DiscoveryPlan, sizes: &BTreeMap<usize, (f64, f64)>) -> Self {
        let (levels, feedback, preorder) = layers(plan);
        let mut rows = vec![Vec::new(); levels.values().max().map_or(0, |level| level + 1)];
        for id in preorder {
            rows[levels[&id]].push(id);
        }
        let mut nodes = BTreeMap::new();
        for row in &rows {
            let mut x = 0.0;
            for &id in row {
                let (width, height) = sizes[&id];
                nodes.insert(
                    id,
                    DiscoveryPlanLayoutNode {
                        x,
                        y: 0.0,
                        width,
                        height,
                    },
                );
                x += width + NODE_SPACING;
            }
        }

        let mut edge_order = plan
            .edges
            .iter()
            .enumerate()
            .filter_map(|(index, edge)| {
                (nodes.contains_key(&edge.source) && nodes.contains_key(&edge.target))
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        edge_order.sort_by(|&a, &b| {
            let a = &plan.edges[a];
            let b = &plan.edges[b];
            (a.source, a.target, &a.label).cmp(&(b.source, b.target, &b.label))
        });
        let mut parents = BTreeMap::<usize, Vec<usize>>::new();
        let mut children = BTreeMap::<usize, Vec<usize>>::new();
        for &index in &edge_order {
            if feedback.contains(&index) {
                continue;
            }
            let edge = &plan.edges[index];
            parents.entry(edge.target).or_default().push(edge.source);
            children.entry(edge.source).or_default().push(edge.target);
        }
        // Fixed passes are deliberate: repeated unconstrained nudges can drift
        // forever. Node ordering is never permuted by this alignment process.
        for _ in 0..ALIGNMENT_PASSES {
            align_downward(&rows, &parents, &mut nodes);
            align_upward(&rows, &children, &mut nodes);
        }

        let right = nodes
            .values()
            .map(|node| node.x + node.width)
            .fold(0.0, f64::max);
        let downward_destinations = edge_order
            .iter()
            .filter_map(|&index| {
                let edge = &plan.edges[index];
                (!feedback.contains(&index) && levels[&edge.target] > levels[&edge.source] + 1)
                    .then_some((levels[&edge.target], edge.target))
            })
            .collect::<BTreeSet<_>>();
        let upward_destinations = edge_order
            .iter()
            .filter_map(|&index| {
                feedback
                    .contains(&index)
                    .then_some((levels[&plan.edges[index].target], plan.edges[index].target))
            })
            .collect::<BTreeSet<_>>();
        let downward_lanes = downward_destinations
            .iter()
            .enumerate()
            .map(|(index, &(_, id))| (id, -LANE_MARGIN - index as f64 * LANE_SPACING))
            .collect::<BTreeMap<_, _>>();
        let upward_lanes = upward_destinations
            .iter()
            .enumerate()
            .map(|(index, &(_, id))| (id, right + LANE_MARGIN + index as f64 * LANE_SPACING))
            .collect::<BTreeMap<_, _>>();

        // Gap 0 is above the first row; gap N is below row N-1. This gives
        // feedback edges and self-loops clear arrival/departure tracks as well.
        let mut gaps = (0..=rows.len()).map(|_| Vec::new()).collect::<Vec<_>>();
        let mut crossings = BTreeMap::new();
        let mut lanes = BTreeMap::new();
        for &index in &edge_order {
            let edge = &plan.edges[index];
            let source = &nodes[&edge.source];
            let target = &nodes[&edge.target];
            let source_x = source.x + source.width / 2.0;
            let target_x = target.x + target.width / 2.0;
            let departure_gap = levels[&edge.source] + 1;
            let arrival_gap = levels[&edge.target];
            let lane = if feedback.contains(&index) {
                Some(upward_lanes[&edge.target])
            } else if arrival_gap != departure_gap {
                Some(downward_lanes[&edge.target])
            } else {
                None
            };
            let departure = (departure_gap, gaps[departure_gap].len());
            gaps[departure_gap].push(DiscoveryPlanLayoutSegment::new(
                source_x,
                lane.unwrap_or(target_x),
                edge.target,
                &edge.label,
            ));
            let arrival = if let Some(lane) = lane {
                lanes.insert(index, lane);
                let arrival = (arrival_gap, gaps[arrival_gap].len());
                gaps[arrival_gap].push(DiscoveryPlanLayoutSegment::new(
                    lane,
                    target_x,
                    edge.target,
                    "",
                ));
                arrival
            } else {
                departure
            };
            crossings.insert(index, [departure, arrival]);
        }

        for gap in &mut gaps {
            // Endpoint stems and the shared dummy runs are all the vertical
            // columns an arrow can occupy. Reserve a clear band for each label
            // before assigning horizontal tracks: text then cannot be crossed
            // by either a vertical trunk or another track's horizontal segment.
            let mut columns = downward_lanes
                .values()
                .chain(upward_lanes.values())
                .copied()
                .chain(
                    gap.iter()
                        .flat_map(|segment| [segment.start_x, segment.end_x]),
                )
                .collect::<Vec<_>>();
            columns.sort_by(f64::total_cmp);
            columns.dedup();
            for segment in gap {
                segment.place_label_between(&columns);
            }
        }
        let gap_heights = gaps
            .iter_mut()
            .map(|segments| assign_tracks(segments))
            .collect::<Vec<_>>();
        let mut y = 0.0;
        for (index, gap) in gaps.iter_mut().enumerate() {
            for segment in gap {
                segment.y += y;
            }
            y += gap_heights[index];
            if let Some(row) = rows.get(index) {
                let mut row_height = 0.0_f64;
                for id in row {
                    let node = nodes.get_mut(id).unwrap();
                    node.y = y;
                    row_height = row_height.max(node.height);
                }
                y += row_height;
            }
        }

        let mut edges = BTreeMap::new();
        for &index in &edge_order {
            let edge = &plan.edges[index];
            let source = &nodes[&edge.source];
            let target = &nodes[&edge.target];
            let [
                (departure_gap, departure_index),
                (arrival_gap, arrival_index),
            ] = crossings[&index];
            let departure = &gaps[departure_gap][departure_index];
            let arrival = &gaps[arrival_gap][arrival_index];
            let source_x = source.x + source.width / 2.0;
            let target_x = target.x + target.width / 2.0;
            let mut points = vec![
                [source_x, source.y + source.height + BINDING_GAP],
                [source_x, departure.y],
            ];
            if let Some(&lane) = lanes.get(&index) {
                points.extend([[lane, departure.y], [lane, arrival.y]]);
            }
            points.extend([[target_x, arrival.y], [target_x, target.y - BINDING_GAP]]);
            simplify(&mut points);
            edges.insert(
                index,
                DiscoveryPlanLayoutEdge {
                    points,
                    label_x: departure.label_x,
                    label_y: departure.y - departure.label_height - 4.0,
                },
            );
        }

        // Normalize the outer dummy runs and their labels along with the real
        // nodes. Geometry is entirely nonnegative before the scene adds margins.
        let mut left = 0.0_f64;
        let mut right = right;
        for gap in &gaps {
            for segment in gap {
                left = left.min(segment.start_x).min(segment.end_x);
                right = right.max(segment.start_x).max(segment.end_x);
                if segment.label_width > 0.0 {
                    left = left.min(segment.label_x);
                    right = right.max(segment.label_x + segment.label_width);
                }
            }
        }
        for node in nodes.values_mut() {
            node.x -= left;
        }
        for edge in edges.values_mut() {
            for point in &mut edge.points {
                point[0] -= left;
            }
            edge.label_x -= left;
        }
        Self {
            nodes,
            edges,
            width: right - left,
            height: y,
        }
    }
}

fn align_downward(
    rows: &[Vec<usize>],
    parents: &BTreeMap<usize, Vec<usize>>,
    nodes: &mut BTreeMap<usize, DiscoveryPlanLayoutNode>,
) {
    for row in rows {
        let mut minimum = 0.0_f64;
        for id in row {
            let desired = neighboring_center(parents.get(id), nodes)
                .map_or(nodes[id].x, |center| center - nodes[id].width / 2.0);
            let node = nodes.get_mut(id).unwrap();
            node.x = desired.max(minimum);
            minimum = node.x + node.width + NODE_SPACING;
        }
    }
}

fn align_upward(
    rows: &[Vec<usize>],
    children: &BTreeMap<usize, Vec<usize>>,
    nodes: &mut BTreeMap<usize, DiscoveryPlanLayoutNode>,
) {
    for row in rows.iter().rev() {
        for (index, id) in row.iter().enumerate().rev() {
            let Some(center) = neighboring_center(children.get(id), nodes) else {
                continue;
            };
            let minimum = index.checked_sub(1).map_or(0.0, |previous| {
                let previous = &nodes[&row[previous]];
                previous.x + previous.width + NODE_SPACING
            });
            let maximum = row.get(index + 1).map_or(f64::INFINITY, |next| {
                nodes[next].x - NODE_SPACING - nodes[id].width
            });
            let node = nodes.get_mut(id).unwrap();
            node.x = (center - node.width / 2.0).max(minimum).min(maximum);
        }
    }
}

fn neighboring_center(
    neighbors: Option<&Vec<usize>>,
    nodes: &BTreeMap<usize, DiscoveryPlanLayoutNode>,
) -> Option<f64> {
    let mut centers = neighbors?
        .iter()
        .map(|id| nodes[id].x + nodes[id].width / 2.0)
        .collect::<Vec<_>>();
    centers.sort_by(f64::total_cmp);
    let middle = centers.len() / 2;
    if centers.len().is_multiple_of(2) {
        Some((centers[middle - 1] + centers[middle]) / 2.0)
    } else {
        Some(centers[middle])
    }
}

fn simplify(points: &mut Vec<[f64; 2]>) {
    points.dedup();
    let mut index = 1;
    while index + 1 < points.len() {
        if (points[index - 1][0] == points[index][0] && points[index][0] == points[index + 1][0])
            || (points[index - 1][1] == points[index][1]
                && points[index][1] == points[index + 1][1])
        {
            points.remove(index);
        } else {
            index += 1;
        }
    }
}
