use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::ExcalidrawLogLevel;
use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::set;
use eyre::Result;
use std::collections::BTreeMap;

pub(crate) fn elements() -> Result<Vec<SceneFields>> {
    let mut elements = vec![
        text(
            "intro",
            "Cloud Terrastodon interactive file\nEdit the answer; move Log Level onto a level.\nSpace in the terminal updates the status below.",
            40.0,
            30.0,
            760.0,
            75.0,
            18.0,
        )?,
        rectangle("question", 40.0, 140.0, 360.0, 130.0)?,
        text(
            "question-label",
            "#UserQuestion1",
            60.0,
            155.0,
            300.0,
            25.0,
            20.0,
        )?,
        text("question-answer", "waffles", 60.0, 205.0, 300.0, 25.0, 20.0)?,
        text(
            "live-status",
            "#CTLiveReload\nWaiting for Space in the terminal",
            440.0,
            160.0,
            400.0,
            60.0,
            18.0,
        )?,
    ];
    let mut frame = rectangle("logs-frame", 40.0, 340.0, 800.0, 420.0)?;
    set(&mut frame, "type", &"frame")?;
    set(&mut frame, "name", &"Logs")?;
    elements.push(frame);
    for (index, level) in [
        ExcalidrawLogLevel::Debug,
        ExcalidrawLogLevel::Info,
        ExcalidrawLogLevel::Warn,
        ExcalidrawLogLevel::Error,
        ExcalidrawLogLevel::Off,
    ]
    .into_iter()
    .enumerate()
    {
        let x = 60.0 + index as f64 * 150.0;
        let mut target = rectangle(&format!("level-{index}"), x, 360.0, 130.0, 85.0)?;
        set(&mut target, "strokeColor", &level.stroke_color())?;
        set(&mut target, "backgroundColor", &level.background_color())?;
        set(&mut target, "fillStyle", &"solid")?;
        set(&mut target, "frameId", &"logs-frame")?;
        elements.push(target);
        let mut label = text(
            &format!("level-label-{index}"),
            level.label(),
            x + 15.0,
            418.0,
            100.0,
            20.0,
            16.0,
        )?;
        set(&mut label, "strokeColor", &level.stroke_color())?;
        set(&mut label, "frameId", &"logs-frame")?;
        elements.push(label);
    }
    let mut selector = rectangle("log-selector", 220.0, 368.0, 110.0, 40.0)?;
    set(&mut selector, "strokeColor", &"#1971c2")?;
    set(&mut selector, "strokeWidth", &3.0)?;
    set(&mut selector, "frameId", &"logs-frame")?;
    set(
        &mut selector,
        "boundElements",
        &vec![BTreeMap::from([
            ("id", "log-selector-label"),
            ("type", "text"),
        ])],
    )?;
    elements.push(selector);
    let mut label = text(
        "log-selector-label",
        "Log Level",
        229.0,
        378.0,
        92.0,
        20.0,
        16.0,
    )?;
    set(&mut label, "containerId", &"log-selector")?;
    set(&mut label, "frameId", &"logs-frame")?;
    elements.push(label);
    let mut output = text("log-output", "#Logs", 60.0, 470.0, 760.0, 270.0, 16.0)?;
    set(&mut output, "frameId", &"logs-frame")?;
    elements.push(output);
    Ok(elements)
}

fn rectangle(id: &str, x: f64, y: f64, width: f64, height: f64) -> Result<SceneFields> {
    let base = ExcalidrawElementBase::new(ExcalidrawElementId(id.into()), x, y, width, height);
    let mut fields = facet_json::from_str(&facet_json::to_string(&base)?)?;
    set(&mut fields, "type", &"rectangle")?;
    Ok(fields)
}

pub(crate) fn text(
    id: &str,
    content: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
) -> Result<SceneFields> {
    let mut fields = rectangle(id, x, y, width, height)?;
    set(&mut fields, "type", &"text")?;
    set(&mut fields, "text", &content)?;
    set(&mut fields, "originalText", &content)?;
    set(&mut fields, "fontSize", &font_size)?;
    set(&mut fields, "fontFamily", &3)?;
    set(&mut fields, "textAlign", &"left")?;
    set(&mut fields, "verticalAlign", &"top")?;
    set(&mut fields, "lineHeight", &1.25)?;
    set(&mut fields, "autoResize", &false)?;
    Ok(fields)
}
