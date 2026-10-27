use super::*;

fn log(message: &str) -> ExcalidrawLogRecord {
    ExcalidrawLogRecord {
        level: ExcalidrawLogLevel::Info,
        message: message.into(),
        fields: vec![("change".into(), "6".into())],
    }
}

fn index(document: &ExcalidrawInteractiveDocument, id: &str) -> usize {
    document
        .elements
        .iter()
        .position(|element| string(element, "id").as_deref() == Some(id))
        .unwrap()
}

#[test]
fn editor_saved_unbound_labels_still_find_their_controls() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    // Excalidraw normalizes absent text bindings to an explicit JSON null.
    // These labels remain spatial controls rather than bound container text.
    for element in &mut document.elements {
        if text(element).is_some() && !element.contains_key("containerId") {
            set(element, "containerId", &Option::<String>::None)?;
        }
    }
    let selector = index(&document, "log-selector");
    let selector_label = index(&document, "log-selector-label");
    set(&mut document.elements[selector], "x", &70.25)?;
    set(&mut document.elements[selector_label], "x", &79.25)?;
    let mut document = ExcalidrawInteractiveDocument::from_json(&document.to_json()?)?;
    assert_eq!(
        document
            .questions()
            .get("#UserQuestion1")
            .map(String::as_str),
        Some("waffles"),
        "{:?}",
        document.diagnostics()
    );
    assert_eq!(
        document.selected_log_level(),
        Some(ExcalidrawLogLevel::Debug)
    );
    assert!(document.diagnostics().is_empty());
    assert!(document.set_logs(&[log("saved controls remain usable")])?);
    Ok(())
}

#[test]
fn controls_use_visible_text_and_ambiguities_never_pick_an_arbitrary_answer() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    assert_eq!(
        document
            .questions()
            .get("#UserQuestion1")
            .map(String::as_str),
        Some("waffles")
    );
    assert_eq!(
        document.selected_log_level(),
        Some(ExcalidrawLogLevel::Info)
    );
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
    let answer = index(&document, "question-answer");
    let mut duplicate = document.elements[answer].clone();
    set(&mut duplicate, "id", &"second-answer")?;
    set(&mut duplicate, "text", &"pancakes")?;
    document.elements.push(duplicate);
    assert!(document.questions().is_empty());
    assert!(
        document
            .diagnostics()
            .iter()
            .any(|message| message.contains("multiple answer"))
    );
    set(document.elements.last_mut().unwrap(), "angle", &0.3)?;
    assert!(document.questions().is_empty());
    assert!(
        document
            .diagnostics()
            .iter()
            .any(|message| message.contains("rotated answer"))
    );
    let duplicate = document.elements.last_mut().unwrap();
    set(duplicate, "isDeleted", &true)?;
    assert_eq!(
        document
            .questions()
            .get("#UserQuestion1")
            .map(String::as_str),
        Some("waffles")
    );
    Ok(())
}

#[test]
fn question_duplicates_and_rotated_containers_are_diagnostic() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    let label = index(&document, "question-label");
    document.elements.push(document.elements[label].clone());
    assert!(document.questions().is_empty());
    assert!(
        document
            .diagnostics()
            .iter()
            .any(|message| message.contains("duplicate visible"))
    );
    document.elements.pop();
    let question = index(&document, "question");
    set(&mut document.elements[question], "angle", &0.5)?;
    assert!(document.questions().is_empty());
    Ok(())
}

#[test]
fn log_selector_acknowledges_valid_selection_and_off_clears_the_body() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    assert!(document.set_logs(&[log("test")])?);
    let selector = index(&document, "log-selector");
    let label = index(&document, "log-selector-label");
    // Move the bound marker and its text together, as Excalidraw does.
    set(&mut document.elements[selector], "x", &670.0)?;
    set(&mut document.elements[label], "x", &679.0)?;
    assert_eq!(document.selected_log_level(), Some(ExcalidrawLogLevel::Off));
    assert_eq!(
        string(&document.elements[selector], "strokeColor").as_deref(),
        Some(ExcalidrawLogLevel::Info.stroke_color()),
        "moving the selector retains the last acknowledged colour until CT publishes"
    );
    let moved_selector = document.elements[selector].clone();
    assert!(document.set_logs(&[log("hidden")])?);
    assert_eq!(
        string(&document.elements[selector], "strokeColor").as_deref(),
        Some(ExcalidrawLogLevel::Off.stroke_color())
    );
    assert_eq!(
        string(&document.elements[selector], "backgroundColor").as_deref(),
        Some(ExcalidrawLogLevel::Off.background_color())
    );
    assert_eq!(
        string(&document.elements[label], "strokeColor").as_deref(),
        Some(ExcalidrawLogLevel::Off.stroke_color())
    );
    for field in ["x", "y", "width", "height", "angle", "boundElements"] {
        assert_eq!(document.elements[selector][field], moved_selector[field]);
    }
    let acknowledged = document.to_json()?;
    assert!(!document.set_logs(&[log("hidden")])?);
    assert_eq!(document.to_json()?, acknowledged);
    let output = index(&document, "log-output");
    assert_eq!(text(&document.elements[output]).as_deref(), Some("#Logs"));
    assert!(
        document
            .elements
            .iter()
            .filter(|element| log_output(element))
            .all(deleted)
    );
    set(&mut document.elements[selector], "x", &800.0)?;
    set(&mut document.elements[label], "x", &809.0)?;
    assert_eq!(document.selected_log_level(), None);
    let before = document.to_json()?;
    assert!(!document.set_logs(&[log("must not write")])?);
    assert_eq!(document.to_json()?, before);
    Ok(())
}

#[test]
fn native_log_parts_keep_full_text_and_noop_keeps_versions() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    let message = "Spacebar requested a drawing update, with a long message and Unicode 界 e\u{301} for Excalidraw to wrap";
    let messages = vec![log(message)];
    assert!(document.set_logs(&messages)?);
    let level = index(&document, "ct-log-log-output-0-level");
    let body = index(&document, "ct-log-log-output-0-message");
    let fields = index(&document, "ct-log-log-output-0-fields");
    assert_eq!(text(&document.elements[body]).as_deref(), Some(message));
    assert_eq!(
        string(&document.elements[body], "originalText").as_deref(),
        Some(message)
    );
    assert_eq!(
        text(&document.elements[fields]).as_deref(),
        Some("change=6")
    );
    assert_eq!(text(&document.elements[level]).as_deref(), Some("INFO"));
    assert_ne!(
        document.elements[level]["strokeColor"],
        document.elements[body]["strokeColor"]
    );
    assert_ne!(
        document.elements[fields]["strokeColor"],
        document.elements[body]["strokeColor"]
    );
    assert_eq!(document.elements[body]["autoResize"].as_str(), "false");
    let frame = index(&document, "logs-frame");
    let bounds = ExcalidrawControlBounds::read(&document.elements[frame]).unwrap();
    for element in [level, body, fields] {
        assert!(
            bounds.contains(ExcalidrawControlBounds::read(&document.elements[element]).unwrap())
        );
    }
    let before = document.to_json()?;
    assert!(!document.set_logs(&messages)?);
    assert_eq!(document.to_json()?, before);
    Ok(())
}

#[test]
fn output_cannot_become_input_controls_and_slot_history_is_bounded() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    let original_count = document.elements.len();
    for iteration in 0..40 {
        let mut records: Vec<_> = (0..30)
            .map(|row| log(&format!("event {iteration}/{row}")))
            .collect();
        records.extend([
            log("#Logs"),
            log("#UserQuestion99"),
            log("Debug"),
            log("Log Level"),
        ]);
        document.set_logs(&records)?;
        assert!(
            document.diagnostics().is_empty(),
            "{:?}",
            document.diagnostics()
        );
        assert_eq!(document.questions().len(), 1);
        assert!(document.elements.len() <= original_count + 16 * 3);
    }
    assert!(
        !document
            .elements
            .iter()
            .any(|element| !deleted(element) && text(element).as_deref() == Some("event 0/0"))
    );
    Ok(())
}

#[test]
fn existing_log_targets_gain_palette_without_moving_user_geometry() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    let target = index(&document, "level-2");
    set(&mut document.elements[target], "strokeColor", &"#000000")?;
    set(
        &mut document.elements[target],
        "backgroundColor",
        &"transparent",
    )?;
    document.elements[target].insert(
        "userData".into(),
        RawJson::from_owned("{\"preserve\":true}".into()),
    );
    let before = document.elements[target].clone();
    document.set_logs(&[])?;
    assert_ne!(
        document.elements[target]["strokeColor"],
        before["strokeColor"]
    );
    for field in ["x", "y", "width", "height", "userData"] {
        assert_eq!(document.elements[target][field], before[field]);
    }
    Ok(())
}

#[test]
fn editing_preserves_opaque_fields_assets_tombstones_and_user_geometry() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    document.fields.insert(
        "futureField".into(),
        RawJson::from_owned("{\"precise\":18446744073709551615,\"nested\":[null,true]}".into()),
    );
    document.fields.insert(
        "files".into(),
        RawJson::from_owned(
            "{\"image\":{\"dataURL\":\"data:image/png;base64,AA==\",\"future\":true}}".into(),
        ),
    );
    let answer = index(&document, "question-answer");
    let mut unknown = document.elements[answer].clone();
    set(&mut unknown, "type", &"future-element-kind")?;
    set(&mut unknown, "isDeleted", &true)?;
    unknown.insert(
        "customData".into(),
        RawJson::from_owned("{\"keep\":[1,2,3]}".into()),
    );
    document.elements.push(unknown.clone());
    let before_fields = document.fields.clone();
    let before_elements = document.elements.clone();
    let serialized = document.to_json()?;
    let mut restored = ExcalidrawInteractiveDocument::from_json(&serialized)?;
    assert!(restored.set_test_status("test update")?);
    assert_eq!(restored.fields, before_fields);
    let status = index(&restored, "live-status");
    for (position, original) in before_elements.iter().enumerate() {
        if position == status {
            for field in ["x", "y", "width", "height", "angle"] {
                assert_eq!(restored.elements[position][field], original[field]);
            }
        } else {
            assert_eq!(&restored.elements[position], original);
        }
    }
    assert!(!restored.set_test_status("test update")?);
    Ok(())
}

#[test]
fn explicit_status_adds_one_marker_without_rebuilding_existing_scene() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    let status = index(&document, "live-status");
    document.elements.remove(status);
    let original = document.elements.clone();
    assert!(document.set_test_status("hello")?);
    assert_eq!(&document.elements[..original.len()], original.as_slice());
    assert_eq!(document.elements.len(), original.len() + 1);
    assert!(!document.set_test_status("hello")?);
    assert!(document.set_test_status("next")?);
    assert_eq!(document.elements.len(), original.len() + 1);
    Ok(())
}

#[test]
fn overflowing_element_version_rejects_update_without_partial_mutation() -> Result<()> {
    let mut document = ExcalidrawInteractiveDocument::demo()?;
    let status = index(&document, "live-status");
    set(&mut document.elements[status], "version", &u64::MAX)?;
    let before = document.to_json()?;
    assert!(document.set_test_status("must not update").is_err());
    assert_eq!(document.to_json()?, before);
    Ok(())
}
