use crate::discovery_plan_layout::DiscoveryPlanLayout;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::DiscoveryPlanEdge;
use cloud_terrastodon_registry::DiscoveryPlanNode;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;
use std::collections::BTreeMap;

const EPSILON: f64 = 0.000_001;

fn graph(ids: &[usize], edges: &[(usize, usize)]) -> DiscoveryPlan {
    DiscoveryPlan {
        nodes: ids
            .iter()
            .map(|&id| DiscoveryPlanNode {
                id,
                label: format!("Node {id}"),
                kind: DiscoveryPlanNodeKind::Shape,
                reachable: true,
                seed: false,
                target: false,
                level: 0,
                effects: Vec::new(),
                fields: Vec::new(),
                missing_inputs: Vec::new(),
            })
            .collect(),
        edges: edges
            .iter()
            .map(|&(source, target)| DiscoveryPlanEdge {
                source,
                target,
                label: format!("{source} to {target}"),
            })
            .collect(),
        diagnostics: Vec::new(),
    }
}

fn sizes(plan: &DiscoveryPlan) -> BTreeMap<usize, (f64, f64)> {
    plan.nodes
        .iter()
        .map(|node| (node.id, (180.0, 80.0)))
        .collect()
}

fn assert_clear_geometry(plan: &DiscoveryPlan, layout: &DiscoveryPlanLayout) {
    assert_eq!(layout.nodes.len(), plan.nodes.len());
    assert_eq!(layout.edges.len(), plan.edges.len());
    assert!(layout.width.is_finite() && layout.width >= 0.0);
    assert!(layout.height.is_finite() && layout.height >= 0.0);

    for (&id, node) in &layout.nodes {
        assert!(node.x.is_finite() && node.y.is_finite());
        assert!(node.width.is_finite() && node.width > 0.0);
        assert!(node.height.is_finite() && node.height > 0.0);
        for (&other_id, other) in layout.nodes.range((id + 1)..) {
            assert!(
                node.x + node.width <= other.x + EPSILON
                    || other.x + other.width <= node.x + EPSILON
                    || node.y + node.height <= other.y + EPSILON
                    || other.y + other.height <= node.y + EPSILON,
                "nodes {id} and {other_id} overlap"
            );
        }
    }

    for (&index, route) in &layout.edges {
        assert!(route.points.len() >= 2, "edge {index} has no route");
        assert!(route.label_x.is_finite() && route.label_y.is_finite());
        for point in &route.points {
            assert!(point[0].is_finite() && point[1].is_finite());
        }
        for segment in route.points.windows(2) {
            let [start, end] = [segment[0], segment[1]];
            assert!(
                (start[0] - end[0]).abs() < EPSILON || (start[1] - end[1]).abs() < EPSILON,
                "edge {index} contains a diagonal segment: {segment:?}"
            );
            for (&id, node) in &layout.nodes {
                assert!(
                    !segment_crosses_interior(
                        start,
                        end,
                        [node.x, node.y, node.width, node.height]
                    ),
                    "edge {index} crosses node {id}: {segment:?}"
                );
            }
        }
    }
}

fn interiors_overlap(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] + a[2] > b[0] + EPSILON
        && b[0] + b[2] > a[0] + EPSILON
        && a[1] + a[3] > b[1] + EPSILON
        && b[1] + b[3] > a[1] + EPSILON
}

fn segment_crosses_interior(start: [f64; 2], end: [f64; 2], bounds: [f64; 4]) -> bool {
    if (start[0] - end[0]).abs() < EPSILON {
        start[0] > bounds[0] + EPSILON
            && start[0] < bounds[0] + bounds[2] - EPSILON
            && start[1].max(end[1]) > bounds[1] + EPSILON
            && start[1].min(end[1]) < bounds[1] + bounds[3] - EPSILON
    } else {
        start[1] > bounds[1] + EPSILON
            && start[1] < bounds[1] + bounds[3] - EPSILON
            && start[0].max(end[0]) > bounds[0] + EPSILON
            && start[0].min(end[0]) < bounds[0] + bounds[2] - EPSILON
    }
}

fn assert_readable_labels(plan: &DiscoveryPlan, layout: &DiscoveryPlanLayout) {
    let labels = plan
        .edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| !edge.label.is_empty())
        .map(|(index, edge)| {
            // Match the renderer's 30-column wrapping and 14px monospace font.
            // Use actual glyph width rather than its wider editor text box.
            let width = edge
                .label
                .lines()
                .map(|line| line.chars().count().min(30))
                .max()
                .unwrap_or_default() as f64
                * 8.4;
            let height = edge
                .label
                .lines()
                .map(|line| line.chars().count().max(1).div_ceil(30))
                .sum::<usize>() as f64
                * 17.5;
            let route = &layout.edges[&index];
            (index, [route.label_x, route.label_y, width, height])
        })
        .collect::<Vec<_>>();
    for (position, &(index, bounds)) in labels.iter().enumerate() {
        for (&id, node) in &layout.nodes {
            assert!(
                !interiors_overlap(bounds, [node.x, node.y, node.width, node.height]),
                "edge {index} label overlaps node {id}"
            );
        }
        for &(other_index, other_bounds) in &labels[position + 1..] {
            assert!(
                !interiors_overlap(bounds, other_bounds),
                "edge labels {index} and {other_index} overlap"
            );
        }
        for (&route_index, route) in &layout.edges {
            for segment in route.points.windows(2) {
                assert!(
                    !segment_crosses_interior(segment[0], segment[1], bounds),
                    "edge {route_index} crosses edge {index} label: {segment:?}"
                );
            }
        }
    }
}

#[test]
fn chain_nodes_align_without_using_planner_reachability_levels() {
    let plan = graph(&[3, 7, 12, 21], &[(3, 7), (7, 12), (12, 21)]);
    let mut dimensions = sizes(&plan);
    dimensions.insert(7, (240.0, 210.0));
    dimensions.insert(12, (320.0, 120.0));
    let layout = DiscoveryPlanLayout::new(&plan, &dimensions);

    assert_clear_geometry(&plan, &layout);
    let center = layout.nodes[&3].x + layout.nodes[&3].width / 2.0;
    for node in layout.nodes.values() {
        assert!((node.x + node.width / 2.0 - center).abs() < EPSILON);
    }
    for edge in &plan.edges {
        let source = &layout.nodes[&edge.source];
        assert!(source.y + source.height < layout.nodes[&edge.target].y);
    }
}

#[test]
fn variable_size_diamond_and_skip_edges_avoid_boxes() {
    let mut plan = graph(
        &[0, 1, 2, 3, 4, 5],
        &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5), (0, 5)],
    );
    plan.edges[0].label = "Organization scope supplied to each request".into();
    plan.edges[1].label = "Authentication context".into();
    plan.edges[2].label = "Pipeline definitions returned by the project request".into();
    plan.edges[3].label = "Optional repository references from build definitions".into();
    plan.edges[6].label =
        "Shared organization input required by the final inventory operation".into();
    let mut dimensions = sizes(&plan);
    dimensions.insert(1, (310.0, 420.0));
    dimensions.insert(2, (220.0, 90.0));
    dimensions.insert(3, (400.0, 150.0));
    let layout = DiscoveryPlanLayout::new(&plan, &dimensions);

    assert_clear_geometry(&plan, &layout);
    assert_readable_labels(&plan, &layout);
    for (&id, &(width, height)) in &dimensions {
        assert_eq!(
            (layout.nodes[&id].width, layout.nodes[&id].height),
            (width, height)
        );
    }
    for edge in &plan.edges {
        let source = &layout.nodes[&edge.source];
        assert!(source.y + source.height < layout.nodes[&edge.target].y);
    }
}

#[test]
fn long_edges_to_the_same_target_share_a_vertical_trunk() {
    let plan = graph(
        &[0, 1, 2, 3, 4],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (0, 4), (1, 4)],
    );
    let layout = DiscoveryPlanLayout::new(&plan, &sizes(&plan));
    assert_clear_geometry(&plan, &layout);
    assert_readable_labels(&plan, &layout);

    let first = &layout.edges[&4].points;
    let second = &layout.edges[&5].points;
    let shared_trunk = first.windows(2).any(|a| {
        (a[0][0] - a[1][0]).abs() < EPSILON
            && second.windows(2).any(|b| {
                (b[0][0] - b[1][0]).abs() < EPSILON
                    && (a[0][0] - b[0][0]).abs() < EPSILON
                    && a[0][1].max(a[1][1]).min(b[0][1].max(b[1][1]))
                        - a[0][1].min(a[1][1]).max(b[0][1].min(b[1][1]))
                        > layout.nodes[&2].height
            })
    });
    assert!(
        shared_trunk,
        "same-target skip edges should share routing space across intermediate layers"
    );
}

#[test]
fn labels_for_distinct_skip_edge_trunks_remain_clear_of_every_route() {
    let mut plan = graph(
        &[0, 1, 2, 3, 4, 5, 6, 7, 8],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (0, 5),
            (1, 6),
            (2, 7),
            (3, 8),
        ],
    );
    for edge in &mut plan.edges[8..] {
        edge.label = "org".into();
    }
    let dimensions = plan
        .nodes
        .iter()
        .map(|node| (node.id, (460.0, 80.0)))
        .collect();
    let layout = DiscoveryPlanLayout::new(&plan, &dimensions);

    assert_clear_geometry(&plan, &layout);
    assert_readable_labels(&plan, &layout);
}

#[test]
fn cycles_and_self_loops_are_routed_outside_all_nodes() {
    let plan = graph(
        &[7, 11, 19, 23, 29],
        &[(7, 11), (11, 19), (19, 7), (11, 11), (7, 23), (23, 29)],
    );
    let mut dimensions = sizes(&plan);
    dimensions.insert(11, (310.0, 300.0));
    dimensions.insert(23, (250.0, 170.0));
    let layout = DiscoveryPlanLayout::new(&plan, &dimensions);

    assert_clear_geometry(&plan, &layout);
    assert!(layout.edges[&3].points.len() > 2);
    assert!(layout.edges[&3].points.windows(2).any(|segment| {
        (segment[0][0] - segment[1][0]).abs() > EPSILON
            || (segment[0][1] - segment[1][1]).abs() > EPSILON
    }));
}

#[test]
fn blocked_and_disconnected_components_retain_their_dependency_layers() {
    let mut plan = graph(&[0, 1, 2, 10, 11, 20], &[(0, 1), (1, 2), (10, 11)]);
    for node in &mut plan.nodes {
        node.reachable = false;
        node.missing_inputs
            .push("Unavailable authentication".into());
    }
    let layout = DiscoveryPlanLayout::new(&plan, &sizes(&plan));

    assert_clear_geometry(&plan, &layout);
    for edge in &plan.edges {
        assert!(layout.nodes[&edge.source].y < layout.nodes[&edge.target].y);
    }
    assert!(
        layout.nodes.contains_key(&20),
        "isolated targets remain visible"
    );
}

#[test]
fn sibling_branches_keep_their_order_when_target_ids_were_allocated_first() {
    let mut plan = graph(
        &[1, 2, 10, 20, 30, 40, 50],
        &[(10, 20), (10, 30), (20, 40), (40, 50), (30, 1), (1, 2)],
    );
    plan.nodes
        .iter_mut()
        .find(|node| node.id == 10)
        .unwrap()
        .seed = true;
    let layout = DiscoveryPlanLayout::new(&plan, &sizes(&plan));

    assert_clear_geometry(&plan, &layout);
    for (left, right) in [(20, 30), (40, 1), (50, 2)] {
        let left_center = layout.nodes[&left].x + layout.nodes[&left].width / 2.0;
        let right_center = layout.nodes[&right].x + layout.nodes[&right].width / 2.0;
        assert!(
            left_center < right_center,
            "sibling branches should preserve their fork order despite target ID allocation"
        );
    }
}

#[test]
fn layout_is_deterministic_independent_of_input_container_order() {
    let plan = graph(
        &[0, 1, 2, 3, 4],
        &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (0, 4), (4, 1)],
    );
    let dimensions = sizes(&plan);
    let original = DiscoveryPlanLayout::new(&plan, &dimensions);
    let repeated = DiscoveryPlanLayout::new(&plan, &dimensions);
    let mut shuffled = plan.clone();
    shuffled.nodes.reverse();
    shuffled.edges.rotate_left(3);
    let reordered = DiscoveryPlanLayout::new(&shuffled, &dimensions);

    for layout in [&repeated, &reordered] {
        assert_eq!(
            (original.width, original.height),
            (layout.width, layout.height)
        );
        for (&id, a) in &original.nodes {
            let b = &layout.nodes[&id];
            assert_eq!((a.x, a.y, a.width, a.height), (b.x, b.y, b.width, b.height));
        }
    }
    for (index, edge) in plan.edges.iter().enumerate() {
        assert_eq!(original.edges[&index].points, repeated.edges[&index].points);
        let new_index = shuffled
            .edges
            .iter()
            .position(|candidate| {
                candidate.source == edge.source
                    && candidate.target == edge.target
                    && candidate.label == edge.label
            })
            .unwrap();
        let a = &original.edges[&index];
        let b = &reordered.edges[&new_index];
        assert_eq!(a.points, b.points);
        assert_eq!((a.label_x, a.label_y), (b.label_x, b.label_y));
    }
}
