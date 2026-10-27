use crate::ExcalidrawArrowBinding;
use crate::ExcalidrawArrowhead;
use crate::ExcalidrawBoundElement;
use crate::ExcalidrawElement;
use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::ExcalidrawStrokeStyle;
use crate::ExcalidrawTextAlignment;
use crate::ExcalidrawVerticalAlignment;
use crate::discovery_plan_layout::DiscoveryPlanLayout;
use crate::discovery_plan_text::EDGE_FONT_SIZE;
use crate::discovery_plan_text::LINE_HEIGHT;
use crate::discovery_plan_text::edge_label_size;
use crate::discovery_plan_text::wrap;
use cloud_terrastodon_registry::DiscoveryPlan;
use cloud_terrastodon_registry::DiscoveryPlanNode;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const NODE_WIDTH: f64 = 460.0;
const FONT_SIZE: f64 = 16.0;
const PADDING: f64 = 16.0;
const BINDING_GAP: f64 = 8.0;

pub(crate) fn elements(plan: &DiscoveryPlan) -> Vec<ExcalidrawElement> {
    let mut result = Vec::new();
    result.push(text(
        ExcalidrawElementBase::new(
            ExcalidrawElementId("plan-title".into()),
            80.0,
            40.0,
            1080.0,
            80.0,
        ),
        "Cloud Terrastodon: offline discovery plan\nBlue: supplied input | Green: target | Red: blocked\nArrows describe dependencies; requests have not been executed.".into(),
        FONT_SIZE,
        None,
    ));

    let node_ids = plan
        .nodes
        .iter()
        .map(|node| node.id)
        .collect::<BTreeSet<_>>();
    let mut attachments = BTreeMap::<usize, Vec<ExcalidrawBoundElement>>::new();
    for (index, edge) in plan.edges.iter().enumerate() {
        if node_ids.contains(&edge.source) && node_ids.contains(&edge.target) {
            attachments
                .entry(edge.source)
                .or_default()
                .push(ExcalidrawBoundElement::Arrow { id: edge_id(index) });
            if edge.source != edge.target {
                attachments
                    .entry(edge.target)
                    .or_default()
                    .push(ExcalidrawBoundElement::Arrow { id: edge_id(index) });
            }
        }
    }
    let labels: BTreeMap<_, _> = plan
        .nodes
        .iter()
        .map(|node| (node.id, node_label(node)))
        .collect();
    let sizes = labels
        .iter()
        .map(|(&id, label)| {
            (
                id,
                (
                    NODE_WIDTH,
                    label.lines().count() as f64 * FONT_SIZE * LINE_HEIGHT + 2.0 * PADDING,
                ),
            )
        })
        .collect();
    let layout = DiscoveryPlanLayout::new(plan, &sizes);
    let mut boxes = BTreeMap::<usize, ExcalidrawElementBase>::new();
    for node in &plan.nodes {
        let placement = &layout.nodes[&node.id];
        let mut base = ExcalidrawElementBase::new(
            node_id(node.id),
            80.0 + placement.x,
            190.0 + placement.y,
            placement.width,
            placement.height,
        );
        base.background_color = if !node.reachable {
            "#ffe3e3"
        } else if node.seed {
            "#d0ebff"
        } else if node.target {
            "#d3f9d8"
        } else if node.kind == DiscoveryPlanNodeKind::Request {
            "#fff3bf"
        } else {
            "#f1f3f5"
        }
        .into();
        if !node.reachable {
            base.stroke_color = "#c92a2a".into();
            base.stroke_style = ExcalidrawStrokeStyle::Dashed;
        }
        base.bound_elements.push(ExcalidrawBoundElement::Text {
            id: label_id(node.id),
        });
        if let Some(arrows) = attachments.remove(&node.id) {
            base.bound_elements.extend(arrows);
        }
        result.push(ExcalidrawElement::Rectangle { base: base.clone() });
        result.push(text(
            ExcalidrawElementBase::new(
                label_id(node.id),
                base.x + PADDING,
                base.y + PADDING,
                base.width - 2.0 * PADDING,
                base.height - 2.0 * PADDING,
            ),
            labels[&node.id].clone(),
            FONT_SIZE,
            Some(base.id.clone()),
        ));
        boxes.insert(node.id, base);
    }

    for (index, edge) in plan.edges.iter().enumerate() {
        let (Some(source), Some(target), Some(route)) = (
            boxes.get(&edge.source),
            boxes.get(&edge.target),
            layout.edges.get(&index),
        ) else {
            // DiscoveryPlan owns endpoint validity. A caller-edited plan with a
            // missing node cannot produce a dangling editor binding.
            continue;
        };
        let Some(&[start_x, start_y]) = route.points.first() else {
            continue;
        };
        let points: Vec<_> = route
            .points
            .iter()
            .map(|point| [point[0] - start_x, point[1] - start_y])
            .collect();
        let (min_x, max_x, min_y, max_y) = points.iter().fold(
            (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
            |(min_x, max_x, min_y, max_y), point| {
                (
                    min_x.min(point[0]),
                    max_x.max(point[0]),
                    min_y.min(point[1]),
                    max_y.max(point[1]),
                )
            },
        );
        result.push(ExcalidrawElement::Arrow {
            base: ExcalidrawElementBase::new(
                edge_id(index),
                80.0 + start_x,
                190.0 + start_y,
                max_x - min_x,
                max_y - min_y,
            ),
            points,
            last_committed_point: None,
            start_binding: Some(ExcalidrawArrowBinding {
                element_id: source.id.clone(),
                focus: 0.0,
                gap: BINDING_GAP,
            }),
            end_binding: Some(ExcalidrawArrowBinding {
                element_id: target.id.clone(),
                focus: 0.0,
                gap: BINDING_GAP,
            }),
            start_arrowhead: None,
            end_arrowhead: Some(ExcalidrawArrowhead::Arrow),
            elbowed: false,
        });
        if !edge.label.is_empty() {
            let label = wrap(&edge.label, 30);
            let (width, height) = edge_label_size(&edge.label);
            result.push(text(
                ExcalidrawElementBase::new(
                    ExcalidrawElementId(format!("edge-label-{index}")),
                    80.0 + route.label_x,
                    190.0 + route.label_y,
                    width,
                    height,
                ),
                label,
                EDGE_FONT_SIZE,
                None,
            ));
        }
    }
    if !plan.diagnostics.is_empty() {
        let label = wrap(&format!("Diagnostics\n{}", plan.diagnostics.join("\n")), 90);
        let height = label.lines().count() as f64 * FONT_SIZE * LINE_HEIGHT;
        result.push(text(
            ExcalidrawElementBase::new(
                ExcalidrawElementId("plan-diagnostics".into()),
                80.0,
                190.0 + layout.height + 80.0,
                900.0,
                height,
            ),
            label,
            FONT_SIZE,
            None,
        ));
    }
    result
}

fn node_label(node: &DiscoveryPlanNode) -> String {
    let kind = match node.kind {
        DiscoveryPlanNodeKind::Shape => "shape",
        DiscoveryPlanNodeKind::Request => "request",
        DiscoveryPlanNodeKind::Conversion => "conversion",
        DiscoveryPlanNodeKind::Projection => "projection",
    };
    let mut label = format!("[{kind}] {}", node.label);
    for field in &node.fields {
        label.push_str(&format!("\n{}: {}", field.name, field.type_name));
    }
    if !node.effects.is_empty() {
        label.push_str(&format!("\nEffects: {:?}", node.effects));
    }
    if node.seed {
        label.push_str("\nSUPPLIED INPUT");
    }
    if node.target {
        label.push_str("\nTARGET");
    }
    if !node.reachable {
        label.push_str("\nBLOCKED");
    }
    if !node.missing_inputs.is_empty() {
        label.push_str("\nMissing: ");
        label.push_str(&node.missing_inputs.join(", "));
    }
    wrap(&label, 44)
}

fn node_id(id: usize) -> ExcalidrawElementId {
    ExcalidrawElementId(format!("node-{id}"))
}

fn label_id(id: usize) -> ExcalidrawElementId {
    ExcalidrawElementId(format!("node-label-{id}"))
}

fn edge_id(id: usize) -> ExcalidrawElementId {
    ExcalidrawElementId(format!("edge-{id}"))
}

fn text(
    base: ExcalidrawElementBase,
    text: String,
    font_size: f64,
    container_id: Option<ExcalidrawElementId>,
) -> ExcalidrawElement {
    ExcalidrawElement::Text {
        base,
        original_text: text.clone(),
        text,
        font_size,
        font_family: 3,
        text_align: ExcalidrawTextAlignment::Left,
        vertical_align: ExcalidrawVerticalAlignment::Top,
        container_id,
        auto_resize: false,
        line_height: LINE_HEIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExcalidrawScene;
    use cloud_terrastodon_registry::DiscoveryPlanEdge;
    use cloud_terrastodon_registry::DiscoveryPlanNode;
    use std::collections::HashSet;

    fn graph() -> DiscoveryPlan {
        DiscoveryPlan {
            nodes: vec![
                DiscoveryPlanNode {
                    id: 2,
                    label: "Organization".into(),
                    kind: DiscoveryPlanNodeKind::Shape,
                    reachable: true,
                    seed: true,
                    target: false,
                    missing_inputs: Vec::new(),
                    effects: Vec::new(),
                    fields: Vec::new(),
                    level: 0,
                },
                DiscoveryPlanNode {
                    id: 5,
                    label: "ListProjects".into(),
                    kind: DiscoveryPlanNodeKind::Request,
                    reachable: true,
                    seed: false,
                    target: true,
                    missing_inputs: Vec::new(),
                    effects: Vec::new(),
                    fields: Vec::new(),
                    level: 1,
                },
            ],
            edges: vec![DiscoveryPlanEdge {
                source: 2,
                target: 5,
                label: "org".into(),
            }],
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn graph_bindings_are_reciprocal_and_ids_unique() {
        let scene = ExcalidrawScene::from_discovery_plan(&graph());
        let ids = scene
            .elements
            .iter()
            .map(|element| &element.base().id)
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), scene.elements.len());
        for element in &scene.elements {
            match element {
                ExcalidrawElement::Arrow {
                    base,
                    start_binding: Some(start),
                    end_binding: Some(end),
                    points,
                    ..
                } => {
                    assert_eq!(points[0], [0.0, 0.0]);
                    assert_eq!(start.element_id, node_id(2));
                    assert_eq!(end.element_id, node_id(5));
                    for id in [&start.element_id, &end.element_id] {
                        let node = scene
                            .elements
                            .iter()
                            .find(|element| &element.base().id == id)
                            .unwrap();
                        assert!(node.base().bound_elements.contains(
                            &ExcalidrawBoundElement::Arrow {
                                id: base.id.clone()
                            }
                        ));
                    }
                }
                ExcalidrawElement::Text {
                    base,
                    container_id: Some(id),
                    ..
                } => {
                    let container = scene
                        .elements
                        .iter()
                        .find(|element| &element.base().id == id)
                        .unwrap();
                    assert!(container.base().bound_elements.contains(
                        &ExcalidrawBoundElement::Text {
                            id: base.id.clone()
                        }
                    ));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn output_is_reproducible_and_survives_wire_roundtrip() -> eyre::Result<()> {
        let plan = graph();
        let scene = ExcalidrawScene::from_discovery_plan(&plan);
        let json = scene.to_json()?;
        assert_eq!(json, ExcalidrawScene::from_discovery_plan(&plan).to_json()?);
        let restored: ExcalidrawScene = facet_json::from_str(&json)?;
        assert_eq!(restored, scene);
        // Tagging/flattening must produce actual scene elements, not nested Rust
        // enum records. The format boundary is required by external editors.
        assert!(json.contains("\"type\": \"rectangle\""));
        assert!(json.contains("\"startBinding\""));
        assert!(!json.contains("\"base\""));
        Ok(())
    }

    #[test]
    fn backward_edges_and_long_unicode_labels_have_finite_geometry() {
        let mut plan = graph();
        plan.nodes[0].label = "长い名前é".repeat(30);
        plan.edges.push(DiscoveryPlanEdge {
            source: 5,
            target: 2,
            label: "back".into(),
        });
        let scene = ExcalidrawScene::from_discovery_plan(&plan);
        for element in &scene.elements {
            let base = element.base();
            assert!(base.x.is_finite() && base.y.is_finite());
            assert!(base.width.is_finite() && base.width >= 0.0);
            assert!(base.height.is_finite() && base.height >= 0.0);
            if !matches!(element, ExcalidrawElement::Arrow { .. }) {
                assert!(base.width > 0.0 && base.height > 0.0);
            }
            if let ExcalidrawElement::Text {
                text,
                container_id: Some(_),
                ..
            } = element
            {
                assert!(text.lines().all(|line| line.chars().count() <= 44));
                assert!(base.height >= text.lines().count() as f64 * FONT_SIZE * LINE_HEIGHT);
            }
        }
    }
}
