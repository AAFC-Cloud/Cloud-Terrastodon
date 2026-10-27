use super::*;
use crate::interactive_scene_fields::number;

fn document() -> ExcalidrawInteractiveDocument {
    let mut document = ExcalidrawInteractiveDocument::demo().unwrap();
    document.elements_mut().clear();
    let mut frame = rectangle(
        "selected",
        "",
        ExcalidrawControlBounds {
            x: 100.0,
            y: 200.0,
            width: 600.0,
            height: 200.0,
        },
    )
    .unwrap();
    set(&mut frame, "type", &"frame").unwrap();
    set(&mut frame, "name", &"Selected targets").unwrap();
    document.elements_mut().push(frame);
    document
}

fn control(document: &ExcalidrawInteractiveDocument) -> (usize, usize) {
    let gate = gates(
        document.elements(),
        "selected",
        bounds(&document.elements()[0]).unwrap(),
    )[0];
    let gate_bounds = bounds(&document.elements()[gate]).unwrap();
    let handle = document
        .elements()
        .iter()
        .position(|element| {
            !deleted(element)
                && purple(element)
                && ExcalidrawControlBounds::read(element)
                    .is_some_and(|child| small_handle(gate_bounds, child))
        })
        .unwrap();
    (gate, handle)
}

fn slide_inside(document: &mut ExcalidrawInteractiveDocument) {
    let (gate, handle) = control(document);
    let gate = bounds(&document.elements()[gate]).unwrap();
    set(&mut document.elements_mut()[handle], "x", &(gate.x + 5.0)).unwrap();
    set(&mut document.elements_mut()[handle], "y", &(gate.y + 5.0)).unwrap();
}

#[test]
fn activation_resets_before_returning_and_stays_acknowledged_after_reload() {
    let mut document = document();
    assert!(
        document
            .ensure_action_control("Selected targets", ExcalidrawCanvasAction::AddTarget)
            .unwrap()
    );
    assert_eq!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap(),
        (false, vec![])
    );
    let before = document.elements().to_vec();
    slide_inside(&mut document);
    assert_eq!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap(),
        (true, vec![ExcalidrawCanvasAction::AddTarget])
    );
    let (gate, handle) = control(&document);
    assert!(
        !bounds(&document.elements()[gate])
            .unwrap()
            .contains(bounds(&document.elements()[handle]).unwrap())
    );
    for (index, original) in before.iter().enumerate() {
        if index != handle {
            assert_eq!(&document.elements()[index], original);
        }
    }
    let saved = document.to_json().unwrap();
    let mut reopened = ExcalidrawInteractiveDocument::from_json(&saved).unwrap();
    assert_eq!(
        reopened
            .reconcile_action_controls("Selected targets")
            .unwrap(),
        (false, vec![])
    );
    assert_eq!(saved, reopened.to_json().unwrap());
}

#[test]
fn hand_drawn_control_does_not_need_generated_ids_and_partial_overlap_does_not_fire() {
    let mut document = document();
    let gate = ExcalidrawControlBounds {
        x: 180.0,
        y: 320.0,
        width: 41.477,
        height: 39.416,
    };
    document
        .elements_mut()
        .push(rectangle("handmade-gate", "selected", gate).unwrap());
    document
        .elements_mut()
        .push(caption("handmade-label", "selected", "Add Target", 237.0, 329.0).unwrap());
    document.elements_mut().push(
        rectangle(
            "handmade-handle",
            "selected",
            ExcalidrawControlBounds {
                x: gate.x - 10.0,
                y: gate.y + 8.0,
                width: 21.206,
                height: 16.639,
            },
        )
        .unwrap(),
    );
    assert!(
        !document
            .ensure_action_control("Selected targets", ExcalidrawCanvasAction::AddTarget)
            .unwrap()
    );
    assert!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap()
            .1
            .is_empty()
    );
    slide_inside(&mut document);
    assert_eq!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap()
            .1,
        vec![ExcalidrawCanvasAction::AddTarget]
    );
}

#[test]
fn missing_caption_and_handle_are_repaired_with_visible_error_and_frame_growth() {
    let mut document = document();
    let gate = ExcalidrawControlBounds {
        x: 180.0,
        y: 358.0,
        width: 42.0,
        height: 42.0,
    };
    document
        .elements_mut()
        .push(rectangle("lonely-gate", "selected", gate).unwrap());
    let annotation = caption("user-note", "selected", "Preserve this note", 180.0, 410.0).unwrap();
    document.elements_mut().push(annotation.clone());
    let (changed, actions) = document
        .reconcile_action_controls("Selected targets")
        .unwrap();
    assert!(changed);
    assert!(actions.is_empty());
    assert_eq!(&document.elements()[2], &annotation);
    assert!(
        document.elements().iter().any(|element| {
            string(element, "originalText").as_deref() == Some("Unknown Action")
        })
    );
    let error = document
        .elements()
        .iter()
        .find(|element| string(element, "strokeColor").as_deref() == Some(ERROR))
        .unwrap();
    assert!(
        bounds(&document.elements()[0])
            .unwrap()
            .contains(bounds(error).unwrap())
    );
    assert!(number(&document.elements()[0], "height").unwrap() > 200.0);
    // Unknown captions remain inert, and acknowledging an invalid gesture
    // prevents editing the caption from unexpectedly executing that old gesture.
    slide_inside(&mut document);
    assert!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap()
            .1
            .is_empty()
    );
    let label = document
        .elements_mut()
        .iter_mut()
        .find(|element| string(element, "originalText").as_deref() == Some("Unknown Action"))
        .unwrap();
    set(label, "text", &"Add Target").unwrap();
    set(label, "originalText", &"Add Target").unwrap();
    assert!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap()
            .1
            .is_empty()
    );
    assert!(document.elements().iter().all(|element| {
        string(element, "strokeColor").as_deref() != Some(ERROR) || deleted(element)
    }));
}

#[test]
fn deleted_or_far_handles_are_reused_without_growing_the_scene_repeatedly() {
    let mut document = document();
    document
        .ensure_action_control("Selected targets", ExcalidrawCanvasAction::AddTarget)
        .unwrap();
    let (gate, handle) = control(&document);
    let count = document.elements().len();
    for deleted_handle in [false, true] {
        set(&mut document.elements_mut()[handle], "x", &9000.0).unwrap();
        set(
            &mut document.elements_mut()[handle],
            "isDeleted",
            &deleted_handle,
        )
        .unwrap();
        let (changed, actions) = document
            .reconcile_action_controls("Selected targets")
            .unwrap();
        assert!(changed);
        assert!(actions.is_empty());
        assert_eq!(document.elements().len(), count);
        assert!(!deleted(&document.elements()[handle]));
        assert!(
            number(&document.elements()[handle], "x").unwrap()
                < number(&document.elements()[gate], "x").unwrap()
        );
        assert_eq!(
            document
                .reconcile_action_controls("Selected targets")
                .unwrap(),
            (false, vec![])
        );
    }
}

#[test]
fn ambiguous_captions_and_handles_do_not_invoke_actions() {
    let mut document = document();
    document
        .ensure_action_control("Selected targets", ExcalidrawCanvasAction::AddTarget)
        .unwrap();
    let (gate, _) = control(&document);
    let gate = bounds(&document.elements()[gate]).unwrap();
    document.elements_mut().push(
        caption(
            "other-label",
            "selected",
            "Add Target",
            gate.x + 58.0,
            gate.y + 8.0,
        )
        .unwrap(),
    );
    slide_inside(&mut document);
    assert!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap()
            .1
            .is_empty()
    );
    assert!(document.elements().iter().any(|element| {
        !deleted(element)
            && string(element, "originalText")
                .is_some_and(|label| label.starts_with("Ambiguous action:"))
    }));
    let other_label = document
        .elements_mut()
        .iter_mut()
        .find(|element| string(element, "id").as_deref() == Some("other-label"))
        .unwrap();
    set(other_label, "isDeleted", &true).unwrap();
    let mut extra = handle("extra", "selected", gate).unwrap();
    set(&mut extra, "x", &(gate.x + 4.0)).unwrap();
    document.elements_mut().push(extra);
    assert!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap()
            .1
            .is_empty()
    );
    assert!(document.elements().iter().any(|element| {
        !deleted(element)
            && string(element, "originalText")
                .is_some_and(|label| label.starts_with("Ambiguous slider:"))
    }));
}

#[test]
fn geometry_and_palette_limit_discovery_to_deliberate_scoped_controls() {
    let mut document = document();
    for (index, (x, width, height, color, angle, owner)) in [
        (800.0, 42.0, 42.0, STROKE, 0.0, "selected"), // outside selected frame
        (180.0, 390.0, 85.0, STROKE, 0.0, "selected"), // target-sized rectangle
        (180.0, 42.0, 42.0, "#1971c2", 0.0, "selected"), // ordinary rectangle
        (180.0, 42.0, 42.0, STROKE, 0.1, "selected"), // rotated square
        (180.0, 42.0, 42.0, STROKE, 0.0, "other-frame"), // overlapping other frame
    ]
    .into_iter()
    .enumerate()
    {
        let mut candidate = rectangle(
            &format!("not-control-{index}"),
            owner,
            ExcalidrawControlBounds {
                x,
                y: 240.0,
                width,
                height,
            },
        )
        .unwrap();
        set(&mut candidate, "strokeColor", &color).unwrap();
        set(&mut candidate, "angle", &angle).unwrap();
        document.elements_mut().push(candidate);
    }
    let before = document.to_json().unwrap();
    assert_eq!(
        document
            .reconcile_action_controls("Selected targets")
            .unwrap(),
        (false, vec![])
    );
    assert_eq!(before, document.to_json().unwrap());
}
