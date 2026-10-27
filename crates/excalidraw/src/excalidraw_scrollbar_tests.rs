use super::*;
use facet_json::RawJson;

fn drawing() -> Vec<SceneFields> {
    let base = ExcalidrawElementBase::new(
        ExcalidrawElementId("search".into()),
        40.0,
        40.0,
        850.0,
        675.0,
    );
    let mut frame: SceneFields =
        facet_json::from_str(&facet_json::to_string(&base).unwrap()).unwrap();
    set(&mut frame, "type", &"frame").unwrap();
    set(&mut frame, "name", &"Target search").unwrap();
    vec![frame]
}

fn page(
    elements: &mut Vec<SceneFields>,
    query: &str,
    total: usize,
    capacity: usize,
) -> (ExcalidrawScrollbar, usize) {
    let mut scrollbar = ExcalidrawScrollbar::prepare(elements, 0, 150.0).unwrap();
    let offset = scrollbar.page(elements, query, total, capacity).unwrap();
    (scrollbar, offset)
}

#[test]
fn dragging_reaches_last_page_snaps_out_of_bounds_and_is_stable_after_reopen() {
    let mut elements = drawing();
    let (scrollbar, offset) = page(&mut elements, "Azure", 637, 10);
    assert_eq!(offset, 0);
    let body = scrollbar.content_bounds();
    assert_eq!(
        (body.x, body.y, body.width, body.height),
        (60.0, 190.0, 810.0, 505.0)
    );
    assert!(scrollbar.changed());
    let thumb = scrollbar.thumb;
    set(&mut elements[thumb], "x", &1200.0).unwrap();
    set(&mut elements[thumb], "y", &1200.0).unwrap();
    set(&mut elements[thumb], "frameId", &Option::<String>::None).unwrap();
    let (scrollbar, offset) = page(&mut elements, "Azure", 637, 10);
    assert_eq!(offset, 627);
    assert_eq!(number(&elements[thumb], "x"), Some(873.0));
    assert_eq!(number(&elements[thumb], "y"), Some(667.0));
    assert_eq!(
        string(&elements[thumb], "frameId").as_deref(),
        Some("search")
    );
    assert!(
        original_text(&elements[scrollbar.status])
            .unwrap()
            .starts_with("#Scroll 628–637 of 637")
    );
    let json = facet_json::to_string(&elements).unwrap();
    let mut reopened = facet_json::from_str(&json).unwrap();
    let (scrollbar, offset) = page(&mut reopened, "Azure", 637, 10);
    assert_eq!(offset, 627);
    assert!(!scrollbar.changed());
    assert_eq!(reopened, elements);
    set(&mut reopened[thumb], "y", &-100.0).unwrap();
    let (_, offset) = page(&mut reopened, "Azure", 637, 10);
    assert_eq!(offset, 0);
    assert_eq!(number(&reopened[thumb], "y"), Some(190.0));
}

#[test]
fn resize_and_result_count_changes_preserve_then_clamp_offset_and_query_resets() {
    let mut elements = drawing();
    let (scrollbar, _) = page(&mut elements, "Azure", 100, 10);
    let travel = scrollbar.length - thumb_length(scrollbar.length, 100, 10);
    set(
        &mut elements[scrollbar.thumb],
        "y",
        &(scrollbar.start + travel / 2.0),
    )
    .unwrap();
    assert_eq!(page(&mut elements, "Azure", 100, 10).1, 45);
    set(&mut elements[0], "height", &1200.0).unwrap();
    assert_eq!(page(&mut elements, "Azure", 100, 20).1, 45);
    assert_eq!(page(&mut elements, "Azure", 50, 20).1, 30);
    assert_eq!(page(&mut elements, "Azure", 8, 20).1, 0);
    assert_eq!(page(&mut elements, "Project", 100, 10).1, 0);
    assert!(!page(&mut elements, "Project", 100, 10).0.changed());
    assert_eq!(page(&mut elements, "Project", 100, 0).1, 0);
    assert!(!page(&mut elements, "Project", 100, 0).0.changed());
    assert_eq!(page(&mut elements, "Nothing", 0, 10).1, 0);
}

#[test]
fn native_numeric_and_text_normalization_does_not_rewrite_idle_controls() {
    let mut elements = drawing();
    let (scrollbar, _) = page(&mut elements, "Azure", 100, 10);
    elements[scrollbar.track].insert(
        "points".into(),
        RawJson::from_owned("[[0,0],[0,505]]".into()),
    );
    elements[scrollbar.track].insert("width".into(), RawJson::from_owned("0".into()));
    elements[scrollbar.track].insert("height".into(), RawJson::from_owned("505".into()));
    set(
        &mut elements[scrollbar.status],
        "text",
        &"#Scroll 1–10 of 100\n(10 slots)\nQuery: Azure",
    )
    .unwrap();
    set(&mut elements[scrollbar.status], "height", &45).unwrap();
    let before = elements.clone();
    assert!(!page(&mut elements, "Azure", 100, 10).0.changed());
    assert_eq!(elements, before);
}

#[test]
fn missing_controls_are_repaired_without_changing_published_offset() {
    let mut elements = drawing();
    let (scrollbar, _) = page(&mut elements, "", 50, 10);
    set(&mut elements[scrollbar.thumb], "y", &2000.0).unwrap();
    let (scrollbar, offset) = page(&mut elements, "", 50, 10);
    assert_eq!(offset, 40);
    set(&mut elements[scrollbar.thumb], "isDeleted", &true).unwrap();
    let (repaired, offset) = page(&mut elements, "", 50, 10);
    assert_eq!(offset, 40);
    assert_ne!(repaired.thumb, scrollbar.thumb);
    set(&mut elements[repaired.track], "isDeleted", &true).unwrap();
    let (_, offset) = page(&mut elements, "", 50, 10);
    assert_eq!(offset, 40);
    assert!(!page(&mut elements, "", 50, 10).0.changed());
}

#[test]
fn line_orientation_and_nearest_edge_support_both_axes_and_reversed_endpoints() {
    let mut elements = drawing();
    let (scrollbar, _) = page(&mut elements, "", 40, 10);
    set(&mut elements[scrollbar.thumb], "y", &2000.0).unwrap();
    assert_eq!(page(&mut elements, "", 40, 10).1, 30);
    set(&mut elements[scrollbar.track], "x", &45.0).unwrap();
    let (scrollbar, offset) = page(&mut elements, "", 40, 10);
    assert_eq!(offset, 30);
    assert!(scrollbar.vertical);
    assert_eq!(scrollbar.cross, 50.0);
    assert_eq!(number(&elements[scrollbar.thumb], "x"), Some(43.0));
    let track = scrollbar.track;
    set(&mut elements[track], "x", &60.0).unwrap();
    set(&mut elements[track], "y", &700.0).unwrap();
    set(&mut elements[track], "width", &810.0).unwrap();
    set(&mut elements[track], "height", &0.0).unwrap();
    set(
        &mut elements[track],
        "points",
        &vec![[810.0, 0.0], [0.0, 0.0]],
    )
    .unwrap();
    let (scrollbar, offset) = page(&mut elements, "", 40, 10);
    assert_eq!(offset, 30);
    assert!(!scrollbar.vertical);
    assert_eq!(scrollbar.cross, 705.0);
    set(&mut elements[scrollbar.thumb], "x", &5000.0).unwrap();
    assert_eq!(page(&mut elements, "", 40, 10).1, 30);
    let (scrollbar, _) = page(&mut elements, "", 40, 10);
    assert!(!scrollbar.changed());
    set(&mut elements[scrollbar.track], "y", &170.0).unwrap();
    let (scrollbar, _) = page(&mut elements, "", 40, 10);
    assert_eq!(scrollbar.cross, 50.0);
    assert_eq!(number(&elements[scrollbar.thumb], "y"), Some(43.0));
    let status = ExcalidrawControlBounds::read(&elements[scrollbar.status]).unwrap();
    assert!(scrollbar.cross + THICKNESS / 2.0 < status.y);
}

#[test]
fn duplicate_visible_controls_are_rejected_without_picking_arbitrarily() {
    let mut elements = drawing();
    let (scrollbar, _) = page(&mut elements, "", 40, 10);
    let mut copy = elements[scrollbar.thumb].clone();
    set(&mut copy, "id", &"pasted-thumb").unwrap();
    elements.push(copy);
    assert!(ExcalidrawScrollbar::prepare(&mut elements, 0, 150.0).is_err());
}
