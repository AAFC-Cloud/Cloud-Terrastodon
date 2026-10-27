use super::*;
use cloud_terrastodon_registry::DiscoveryPlanEdge;
use cloud_terrastodon_registry::DiscoveryPlanNode;
use cloud_terrastodon_registry::DiscoveryPlanNodeKind;

fn board() -> Result<ExcalidrawPlanningBoard> {
    ExcalidrawPlanningBoard::initialize(
        None,
        &names(),
        &["Project".into()],
        "Organization: example",
    )
}

fn names() -> Vec<String> {
    vec!["Project".into(), "Repository".into()]
}

#[test]
fn diagram_only_migrates_generated_controls_and_preserves_annotations() -> Result<()> {
    let mut old = board()?;
    old.set_plan(&plan(false))?;
    let [pool, selected, _] = old.frames()?;
    move_card(&mut old, "Repository", pool, selected)?;
    let selected_id = required_id(&old.document.elements()[selected])?;
    let mut annotation = crate::interactive_demo::text(
        "user-note",
        "Keep my note",
        1000.0,
        100.0,
        240.0,
        30.0,
        16.0,
    )?;
    set(&mut annotation, "frameId", &selected_id)?;
    old.document.elements_mut().push(annotation);
    let mut migrated = ExcalidrawPlanningBoard::initialize_diagram(Some(&old.to_json()?))?;
    assert_eq!(
        migrated
            .document
            .elements()
            .iter()
            .filter(|element| {
                !deleted(element) && string(element, "type").as_deref() == Some("frame")
            })
            .map(|element| string(element, "name").unwrap())
            .collect::<Vec<_>>(),
        [GENERATED],
    );
    assert!(!migrated.document.elements().iter().any(|element| {
        !deleted(element)
            && original_text(element).is_some_and(|text| text.starts_with("#Target\n"))
    }));
    let note = migrated
        .document
        .elements()
        .iter()
        .find(|element| string(element, "id").as_deref() == Some("user-note"))
        .unwrap();
    assert!(!deleted(note));
    assert_eq!(original_text(note).as_deref(), Some("Keep my note"));
    assert!(string(note, "frameId").is_none());
    migrated.set_plan(&plan(true))?;
    assert!(
        migrated
            .document
            .elements()
            .iter()
            .filter(|element| { !deleted(element) && string(element, "frameId").is_some() })
            .all(|element| element
                .get("groupIds")
                .is_none_or(|groups| groups.as_str() == "[]"))
    );
    let stable = migrated.to_json()?;
    let mut reopened = ExcalidrawPlanningBoard::initialize_diagram(Some(&stable))?;
    assert!(!reopened.set_plan(&plan(true))?);
    assert_eq!(reopened.to_json()?, stable);
    Ok(())
}

fn plan(include_repository: bool) -> DiscoveryPlan {
    let mut nodes = vec![DiscoveryPlanNode {
        id: 0,
        label: "Project".into(),
        kind: DiscoveryPlanNodeKind::Shape,
        reachable: true,
        seed: true,
        target: !include_repository,
        missing_inputs: Vec::new(),
        effects: Vec::new(),
        fields: Vec::new(),
        level: 0,
    }];
    if include_repository {
        nodes.push(DiscoveryPlanNode {
            id: 1,
            label: "Repository".into(),
            kind: DiscoveryPlanNodeKind::Shape,
            reachable: true,
            seed: false,
            target: true,
            missing_inputs: Vec::new(),
            effects: Vec::new(),
            fields: Vec::new(),
            level: 1,
        });
    }
    DiscoveryPlan {
        nodes,
        edges: if include_repository {
            vec![DiscoveryPlanEdge {
                source: 0,
                target: 1,
                label: "repository".into(),
            }]
        } else {
            Vec::new()
        },
        diagnostics: Vec::new(),
    }
}

fn move_card(
    board: &mut ExcalidrawPlanningBoard,
    name: &str,
    from: usize,
    to: usize,
) -> Result<()> {
    let from = bounds(&board.document.elements()[from])?;
    let destination = bounds(&board.document.elements()[to])?;
    let (_, label, rectangle) = board
        .cards()?
        .into_iter()
        .find(|(target, _, rectangle)| {
            target == name
                && from.contains_center(bounds(&board.document.elements()[*rectangle]).unwrap())
        })
        .unwrap();
    let original = bounds(&board.document.elements()[rectangle])?;
    for index in [label, rectangle] {
        let element = &mut board.document.elements_mut()[index];
        set(
            element,
            "x",
            &(number(element, "x").unwrap() + destination.x + 25.0 - original.x),
        )?;
        set(
            element,
            "y",
            &(number(element, "y").unwrap() + destination.y + 180.0 - original.y),
        )?;
        // Simulate editor-bound frame membership independently from position.
        set(element, "frameId", &Option::<String>::None)?;
    }
    Ok(())
}

fn cards_in(board: &ExcalidrawPlanningBoard, frame: usize, name: &str) -> usize {
    let frame = bounds(&board.document.elements()[frame]).unwrap();
    board
        .cards()
        .unwrap()
        .iter()
        .filter(|(target, _, rectangle)| {
            target == name
                && frame.contains_center(bounds(&board.document.elements()[*rectangle]).unwrap())
        })
        .count()
}

#[test]
fn dragging_from_pool_replenishes_and_acknowledges_once() -> Result<()> {
    let mut board = board()?;
    let [pool, selected, _] = board.frames()?;
    move_card(&mut board, "Repository", pool, selected)?;
    assert_eq!(board.selected_targets()?, names());
    assert!(board.reconcile_pool(&names())?);
    assert_eq!(cards_in(&board, pool, "Repository"), 1);
    assert_eq!(cards_in(&board, selected, "Repository"), 1);
    assert!(!board.reconcile_pool(&names())?);
    for (name, _, rectangle) in board.cards()? {
        if name == "Repository"
            && bounds(&board.document.elements()[pool])?
                .contains_center(bounds(&board.document.elements()[rectangle])?)
        {
            assert_eq!(
                string(&board.document.elements()[rectangle], "backgroundColor").as_deref(),
                Some("#d3f9d8")
            );
        }
    }
    move_card(&mut board, "Repository", selected, pool)?;
    assert!(board.reconcile_pool(&names())?);
    assert_eq!(cards_in(&board, pool, "Repository"), 1);
    assert_eq!(board.selected_targets()?, vec!["Project"]);
    assert!(!board.reconcile_pool(&names())?);
    Ok(())
}

#[test]
fn initialization_preserves_existing_drawing_and_empty_selection_on_restart() -> Result<()> {
    let mut drawing = ExcalidrawInteractiveDocument::demo()?;
    let mut unknown = rectangle("future-element", -200.0, -400.0, 10.0, 10.0)?;
    set(&mut unknown, "futureField", &"opaque")?;
    drawing.elements_mut().push(unknown);
    let originals = drawing.elements().to_vec();
    let mut board =
        ExcalidrawPlanningBoard::initialize(Some(&drawing.to_json()?), &names(), &[], "example")?;
    assert_eq!(&board.document.elements()[..originals.len()], originals);
    board.set_plan(&plan(false))?;
    let before = board.to_json()?;
    let mut reopened = ExcalidrawPlanningBoard::initialize(
        Some(&before),
        &names(),
        &["Project".into()],
        "example",
    )?;
    assert!(reopened.selected_targets()?.is_empty());
    assert!(!reopened.set_plan(&plan(false))?);
    assert_eq!(reopened.to_json()?, before);
    Ok(())
}

#[test]
fn generated_graph_remaps_bindings_preserves_opaque_fields_and_reuses_tombstones() -> Result<()> {
    let original = ExcalidrawScene::from_discovery_plan(&plan(true)).to_json()?;
    let mut board = ExcalidrawPlanningBoard::initialize(Some(&original), &names(), &[], "example")?;
    board.set_plan(&plan(true))?;
    let generated = board.frames()?[2];
    let frame_id = required_id(&board.document.elements()[generated])?;
    let prefix = format!("{frame_id}-plan-");
    let ids: BTreeSet<_> = board
        .document
        .elements()
        .iter()
        .map(required_id)
        .collect::<Result<_>>()?;
    assert_eq!(ids.len(), board.document.elements().len());
    for element in board
        .document
        .elements()
        .iter()
        .filter(|element| owns_output(element, &prefix))
    {
        assert_eq!(
            string(element, "frameId").as_deref(),
            Some(frame_id.as_str())
        );
        for binding in ["startBinding", "endBinding"] {
            if let Some(value) = element.get(binding) {
                let value: SceneFields = facet_json::from_str(value.as_str())?;
                let id = string(&value, "elementId").unwrap();
                assert!(id.starts_with(&prefix) && ids.contains(&id));
            }
        }
        if let Some(container) = string(element, "containerId") {
            assert!(container.starts_with(&prefix) && ids.contains(&container));
        }
    }
    let node = board
        .document
        .elements()
        .iter()
        .position(|element| string(element, "id") == Some(format!("{prefix}node-1")))
        .unwrap();
    set(
        &mut board.document.elements_mut()[node],
        "futureField",
        &"keep this",
    )?;
    let count = board.document.elements().len();
    let initial_version = number(&board.document.elements()[node], "version").unwrap();
    for _ in 0..4 {
        assert!(board.set_plan(&plan(false))?);
        assert!(deleted(&board.document.elements()[node]));
        assert!(board.set_plan(&plan(true))?);
        assert!(!deleted(&board.document.elements()[node]));
    }
    assert_eq!(board.document.elements().len(), count);
    assert_eq!(
        string(&board.document.elements()[node], "futureField").as_deref(),
        Some("keep this")
    );
    assert!(number(&board.document.elements()[node], "version").unwrap() > initial_version);
    Ok(())
}

#[test]
fn malformed_or_ambiguous_controls_do_not_publish_partial_updates() -> Result<()> {
    let mut board = board()?;
    let [pool, selected, _] = board.frames()?;
    let position = bounds(&board.document.elements()[pool])?;
    set(
        &mut board.document.elements_mut()[selected],
        "x",
        &position.x,
    )?;
    set(
        &mut board.document.elements_mut()[selected],
        "y",
        &position.y,
    )?;
    let before = board.to_json()?;
    assert!(board.selected_targets().is_err());
    assert!(board.reconcile_pool(&names()).is_err());
    assert_eq!(before, board.to_json()?);
    let duplicate = board.document.elements()[pool].clone();
    board.document.elements_mut().push(duplicate);
    assert!(
        board
            .selected_targets()
            .unwrap_err()
            .to_string()
            .contains("exactly one")
    );
    Ok(())
}

#[test]
fn text_wrapping_and_copied_element_ids_do_not_change_target_identity() -> Result<()> {
    let mut board = board()?;
    let [pool, selected, _] = board.frames()?;
    move_card(&mut board, "Repository", pool, selected)?;
    let (_, label, rectangle) = board
        .cards()?
        .into_iter()
        .find(|(name, _, rectangle)| {
            name == "Repository"
                && bounds(&board.document.elements()[selected])
                    .unwrap()
                    .contains_center(bounds(&board.document.elements()[*rectangle]).unwrap())
        })
        .unwrap();
    set(
        &mut board.document.elements_mut()[rectangle],
        "id",
        &"pasted-rectangle",
    )?;
    set(
        &mut board.document.elements_mut()[label],
        "id",
        &"pasted-text",
    )?;
    set(
        &mut board.document.elements_mut()[label],
        "containerId",
        &"pasted-rectangle",
    )?;
    set(
        &mut board.document.elements_mut()[label],
        "text",
        &"#Target\nRepos\nitory",
    )?;
    assert_eq!(board.selected_targets()?, names());
    board.reconcile_pool(&names())?;
    assert_eq!(cards_in(&board, pool, "Repository"), 1);
    Ok(())
}

#[test]
fn registry_sized_catalog_has_bounded_results_and_reopening_preserves_unknown_targets() -> Result<()>
{
    let available: Vec<_> = (0..200)
        .map(|index| format!("DomainType{index:03}"))
        .collect();
    let mut board = ExcalidrawPlanningBoard::initialize(
        None,
        &available,
        &["FromEarlierRegistry".into()],
        "Organization: before",
    )?;
    let [pool, _, _] = board.frames()?;
    assert_eq!(
        board
            .cards()?
            .iter()
            .filter(|(_, _, index)| bounds(&board.document.elements()[pool])
                .unwrap()
                .contains_center(bounds(&board.document.elements()[*index]).unwrap()))
            .count(),
        10
    );
    assert!(!board.reconcile_pool(&available)?);
    let reopened = ExcalidrawPlanningBoard::initialize(
        Some(&board.to_json()?),
        &available,
        &[],
        "Organization: after",
    )?;
    assert_eq!(reopened.selected_targets()?, vec!["FromEarlierRegistry"]);
    assert!(
        reopened
            .document
            .elements()
            .iter()
            .filter_map(original_text)
            .any(|label| label.contains("Organization: after"))
    );
    Ok(())
}

#[test]
fn empty_search_uses_recommendation_order_and_independent_card_groups() -> Result<()> {
    let available: Vec<String> = ["Zebra", "Alpha", "Zebra", "Beta"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let board = ExcalidrawPlanningBoard::initialize(
        None,
        &available,
        &["UnknownZ".into(), "UnknownA".into()],
        "example",
    )?;
    let pool = bounds(&board.document.elements()[board.frames()?[0]])?;
    let mut positions: Vec<_> = board
        .cards()?
        .into_iter()
        .filter_map(|(name, _, index)| {
            let position = bounds(&board.document.elements()[index]).unwrap();
            pool.contains_center(position).then_some((name, position))
        })
        .collect();
    positions.sort_by(|(_, left), (_, right)| {
        left.y.total_cmp(&right.y).then(left.x.total_cmp(&right.x))
    });
    assert_eq!(
        positions
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["Zebra", "Alpha", "Beta", "UnknownA", "UnknownZ"]
    );
    let groups: BTreeSet<_> = board
        .cards()?
        .into_iter()
        .filter(|(_, _, index)| {
            pool.contains_center(bounds(&board.document.elements()[*index]).unwrap())
        })
        .map(|(_, _, index)| {
            board.document.elements()[index]["groupIds"]
                .as_str()
                .to_owned()
        })
        .collect();
    assert_eq!(
        groups.len(),
        5,
        "dragging a result must not select every result"
    );
    let before = board.to_json()?;
    let mut reopened =
        ExcalidrawPlanningBoard::initialize(Some(&before), &available, &[], "example")?;
    assert!(!reopened.reconcile_pool(&available)?);
    assert_eq!(reopened.to_json()?, before);
    Ok(())
}

fn set_query(board: &mut ExcalidrawPlanningBoard, query: &str) -> Result<()> {
    let element = board
        .document
        .elements_mut()
        .iter_mut()
        .find(|element| {
            original_text(element).is_some_and(|text| text.lines().next() == Some("#target"))
        })
        .unwrap();
    let text = format!("#target\n{query}");
    set(element, "text", &text)?;
    set(element, "originalText", &text)
}

fn search_names(board: &ExcalidrawPlanningBoard) -> Result<Vec<String>> {
    let pool = bounds(&board.document.elements()[board.frames()?[0]])?;
    let mut cards: Vec<_> = board
        .cards()?
        .into_iter()
        .filter_map(|(name, _, index)| {
            let card = bounds(&board.document.elements()[index]).unwrap();
            pool.contains_center(card).then_some((name, card))
        })
        .collect();
    cards.sort_by(|(_, left), (_, right)| {
        left.y.total_cmp(&right.y).then(left.x.total_cmp(&right.x))
    });
    Ok(cards.into_iter().map(|(name, _)| name).collect())
}

fn scroll_to_end(board: &mut ExcalidrawPlanningBoard) -> Result<()> {
    let pool = bounds(&board.document.elements()[board.frames()?[0]])?;
    let thumb = board
        .document
        .elements_mut()
        .iter_mut()
        .find(|element| {
            !deleted(element)
                && string(element, "type").as_deref() == Some("rectangle")
                && string(element, "strokeColor").as_deref() == Some("#7048e8")
        })
        .unwrap();
    // A native drag can leave the frame and lose its frame membership. CT
    // should clamp it back onto the track while retaining the requested page.
    set(thumb, "x", &(pool.x + pool.width + 50.0))?;
    set(thumb, "y", &(pool.y + pool.height + 50.0))?;
    set(thumb, "frameId", &Option::<String>::None)
}

#[test]
fn scrolling_reuses_slots_and_preserves_dragged_selection_and_reopened_offset() -> Result<()> {
    let names: Vec<_> = (0..60).map(|index| format!("Target{index:02}")).collect();
    let mut board = ExcalidrawPlanningBoard::initialize(None, &names, &[], "example")?;
    let original_count = board.document.elements().len();
    scroll_to_end(&mut board)?;
    assert!(board.reconcile_pool(&names)?);
    assert_eq!(search_names(&board)?, names[50..]);
    assert_eq!(board.document.elements().len(), original_count);
    assert!(!board.reconcile_pool(&names)?);

    let json = board.to_json()?;
    let mut reopened = ExcalidrawPlanningBoard::initialize(Some(&json), &names, &[], "example")?;
    assert_eq!(reopened.to_json()?, json);
    let [pool, selected, _] = reopened.frames()?;
    move_card(&mut reopened, &names[55], pool, selected)?;
    assert!(reopened.reconcile_pool(&names)?);
    assert_eq!(reopened.selected_targets()?, [names[55].clone()]);
    assert_eq!(search_names(&reopened)?, names[50..]);
    let pool_bounds = bounds(&reopened.document.elements()[pool])?;
    let (_, _, acknowledged) = reopened
        .cards()?
        .into_iter()
        .find(|(name, _, rectangle)| {
            name == &names[55]
                && pool_bounds
                    .contains_center(bounds(&reopened.document.elements()[*rectangle]).unwrap())
        })
        .unwrap();
    assert_eq!(
        string(
            &reopened.document.elements()[acknowledged],
            "backgroundColor"
        )
        .as_deref(),
        Some("#d3f9d8")
    );
    assert!(!reopened.reconcile_pool(&names)?);

    set_query(&mut reopened, "Target00")?;
    reopened.reconcile_pool(&names)?;
    assert_eq!(search_names(&reopened)?, [names[0].clone()]);
    assert_eq!(reopened.selected_targets()?, [names[55].clone()]);
    set_query(&mut reopened, "")?;
    reopened.reconcile_pool(&names)?;
    assert_eq!(search_names(&reopened)?, names[..10]);
    Ok(())
}

#[test]
fn resizing_scrolled_results_clamps_to_a_full_last_page() -> Result<()> {
    let names: Vec<_> = (0..30).map(|index| format!("Target{index:02}")).collect();
    let mut board = ExcalidrawPlanningBoard::initialize(None, &names, &[], "example")?;
    scroll_to_end(&mut board)?;
    board.reconcile_pool(&names)?;
    assert_eq!(search_names(&board)?, names[20..]);

    let pool = board.frames()?[0];
    set(
        &mut board.document.elements_mut()[pool],
        "height",
        &(HEADER_HEIGHT + 10.0 * (CARD_HEIGHT + GAP)),
    )?;
    board.reconcile_pool(&names)?;
    assert_eq!(search_names(&board)?, names[10..]);
    assert!(!board.reconcile_pool(&names)?);
    set(
        &mut board.document.elements_mut()[pool],
        "height",
        &SEARCH_HEIGHT,
    )?;
    board.reconcile_pool(&names)?;
    assert_eq!(search_names(&board)?, names[10..20]);
    assert!(!board.reconcile_pool(&names)?);
    Ok(())
}

#[test]
fn fuzzy_query_and_resized_frame_control_reusable_result_slots() -> Result<()> {
    let mut names: Vec<String> = [
        "Resource",
        "BuildDefinitionReference",
        "BuildDefinition",
        "BuildFolder",
        "AzureDevOpsBuildDefinition",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    names.extend((0..30).map(|index| format!("OtherType{index}")));
    let mut board = ExcalidrawPlanningBoard::initialize(None, &names, &[], "example")?;
    assert_eq!(search_names(&board)?.len(), 10);
    let original_count = board.document.elements().len();
    set_query(&mut board, "blddef")?;
    assert!(board.reconcile_pool(&names)?);
    assert_eq!(
        search_names(&board)?.first().map(String::as_str),
        // Both BuildDefinition prefixes have the same fuzzy score here; the
        // caller recommended the Reference form first, so preserve that tie.
        Some("BuildDefinitionReference")
    );
    assert_eq!(
        search_names(&board)?.into_iter().collect::<BTreeSet<_>>(),
        [
            "BuildDefinition",
            "BuildDefinitionReference",
            "AzureDevOpsBuildDefinition"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    assert_eq!(board.document.elements().len(), original_count);
    assert!(!board.reconcile_pool(&names)?);
    set_query(&mut board, "nothing_matches_this")?;
    board.reconcile_pool(&names)?;
    assert!(search_names(&board)?.is_empty());
    set_query(&mut board, "")?;
    let pool = board.frames()?[0];
    set(
        &mut board.document.elements_mut()[pool],
        "width",
        &(GAP + CARD_WIDTH + GAP),
    )?;
    set(
        &mut board.document.elements_mut()[pool],
        "height",
        &(HEADER_HEIGHT + 3.0 * (CARD_HEIGHT + GAP)),
    )?;
    board.reconcile_pool(&names)?;
    assert_eq!(search_names(&board)?.len(), 3);
    assert_eq!(board.document.elements().len(), original_count);
    assert!(!board.reconcile_pool(&names)?);
    Ok(())
}

#[test]
fn picker_replacement_preserves_retained_geometry_and_action_controls() -> Result<()> {
    let mut board = board()?;
    let selected = board.frames()?[1];
    let area = bounds(&board.document.elements()[selected])?;
    let (_, _, project) = board
        .cards()?
        .into_iter()
        .find(|(name, _, index)| {
            name == "Project"
                && area.contains_center(bounds(&board.document.elements()[*index]).unwrap())
        })
        .unwrap();
    let existing = board.document.elements()[project].clone();
    let action = rectangle(
        "action-control",
        area.x + GAP + CARD_WIDTH + GAP,
        area.y + HEADER_HEIGHT,
        CARD_WIDTH,
        CARD_HEIGHT,
    )?;
    board.document.elements_mut().push(action.clone());
    assert!(board.replace_selected_targets(&["Project".into(), "Repository".into()])?);
    assert_eq!(board.document.elements()[project], existing);
    assert_eq!(
        board
            .document
            .elements()
            .iter()
            .find(|element| string(element, "id").as_deref() == Some("action-control")),
        Some(&action)
    );
    assert_eq!(board.selected_targets()?, names());
    assert!(!board.replace_selected_targets(&names())?);
    assert!(board.replace_selected_targets(&["Repository".into()])?);
    assert!(deleted(&board.document.elements()[project]));
    assert_eq!(board.selected_targets()?, vec!["Repository"]);
    assert!(board.replace_selected_targets(&[])?);
    assert!(board.selected_targets()?.is_empty());
    Ok(())
}

#[test]
fn legacy_inventory_migrates_once_without_deleting_moved_cards_or_annotations() -> Result<()> {
    let mut board = board()?;
    let [pool, _, _] = board.frames()?;
    let area = bounds(&board.document.elements()[pool])?;
    // Recreate an old inventory from typed drawing primitives: these cards have
    // no search ownership groups and the frame is the former large palette.
    for element in board.document.elements_mut() {
        if original_text(element).is_some_and(|text| text.lines().next() == Some("#target"))
            || owns_search_result(element, &format!("ct-planning-frame-0-target-results"))
        {
            set(element, "isDeleted", &true)?;
        }
    }
    set(
        &mut board.document.elements_mut()[pool],
        "name",
        &LEGACY_POOL,
    )?;
    set(&mut board.document.elements_mut()[pool], "height", &5000.0)?;
    let (inventory_label, inventory) =
        board.add_card("Repository", pool, area.x + GAP, area.y + HEADER_HEIGHT)?;
    let (moved_label, moved) = board.add_card("UserMoved", pool, area.x + 37.0, area.y + 350.0)?;
    let moved_before = board.document.elements()[moved].clone();
    let moved_label_before = board.document.elements()[moved_label].clone();
    let mut note = crate::interactive_demo::text(
        "user-note",
        "Keep my annotation",
        area.x + 600.0,
        area.y + 2000.0,
        180.0,
        30.0,
        16.0,
    )?;
    set(
        &mut note,
        "frameId",
        &required_id(&board.document.elements()[pool])?,
    )?;
    set(&mut note, "opaqueAnnotationProperty", &"preserved")?;
    board.document.elements_mut().push(note.clone());
    // The note remains at the same world position, but must be detached from
    // the resized native frame or Excalidraw would clip it out of view.
    set(&mut note, "frameId", &Option::<String>::None)?;
    let mut migrated = ExcalidrawPlanningBoard::initialize(
        Some(&board.to_json()?),
        &names(),
        &[],
        "Organization: example",
    )?;
    assert!(deleted(&migrated.document.elements()[inventory]));
    assert!(deleted(&migrated.document.elements()[inventory_label]));
    assert_eq!(migrated.document.elements()[moved], moved_before);
    assert_eq!(
        migrated.document.elements()[moved_label],
        moved_label_before
    );
    let migrated_note = migrated
        .document
        .elements()
        .iter()
        .find(|element| string(element, "id").as_deref() == Some("user-note"))
        .unwrap();
    for (key, value) in &note {
        if !["version", "versionNonce", "updated"].contains(&key.as_str()) {
            assert_eq!(migrated_note.get(key), Some(value), "{key}");
        }
    }
    assert_eq!(
        bounds(&migrated.document.elements()[pool])?.height,
        SEARCH_HEIGHT
    );
    assert_eq!(migrated.target_query()?, "");
    let before = migrated.to_json()?;
    migrated.migrate_legacy_pool()?;
    assert!(!migrated.reconcile_pool(&names())?);
    assert_eq!(migrated.to_json()?, before);
    Ok(())
}
