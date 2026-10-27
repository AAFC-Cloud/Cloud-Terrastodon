//! Native canvas log elements and their bounded lifecycle. Text is never split
//! into lines here: Excalidraw owns font measurement and wrapping in the editor.

use crate::ExcalidrawLogLevel;
use crate::ExcalidrawLogRecord;
use crate::excalidraw_control_bounds::ExcalidrawControlBounds;
use crate::excalidraw_row_layout::ExcalidrawRowLayout;
use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::edit;
use crate::interactive_scene_fields::set;
use crate::interactive_scene_fields::string;
use eyre::Result;
use std::collections::BTreeSet;

const MAX_ROWS: usize = 16;
const GAP: f64 = 8.0;
const BOTTOM_PADDING: f64 = 20.0;

pub(crate) fn update(
    elements: &mut Vec<SceneFields>,
    frame: usize,
    output: usize,
    level: ExcalidrawLogLevel,
    records: &[ExcalidrawLogRecord],
) -> Result<bool> {
    let frame_bounds = ExcalidrawControlBounds::read(&elements[frame]).unwrap();
    let bounds = ExcalidrawControlBounds::read(&elements[output]).unwrap();
    let frame_id = string(&elements[frame], "id")
        .ok_or_else(|| eyre::eyre!("Logs frame has no element ID"))?;
    let output_id = string(&elements[output], "id")
        .ok_or_else(|| eyre::eyre!("#Logs label has no element ID"))?;
    let font_size =
        crate::interactive_scene_fields::number(&elements[output], "fontSize").unwrap_or(16.0);
    if !font_size.is_finite() || font_size <= 0.0 {
        return Ok(false);
    }
    let row_height = font_size * 1.25;
    let width = bounds
        .width
        .min(frame_bounds.x + frame_bounds.width - bounds.x);
    let top = bounds.y - frame_bounds.y + row_height + GAP;
    let available_height = (frame_bounds.height - top - BOTTOM_PADDING).max(0.0);
    // This is only an upper bound on candidate rows, using a one-line minimum.
    // The renderer chooses the rows that actually fit after native measurement.
    let capacity = ((available_height + GAP) / (row_height + GAP))
        .floor()
        .clamp(0.0, MAX_ROWS as f64) as usize;
    let records = if level == ExcalidrawLogLevel::Off || width < font_size * 10.0 {
        &records[..0]
    } else {
        &records[records.len().saturating_sub(capacity)..]
    };
    let mut changed = false;
    changed |= edit(&mut elements[output], |element| {
        set(element, "text", &"#Logs")?;
        set(element, "originalText", &"#Logs")?;
        set(element, "height", &row_height)?;
        set(element, "strokeColor", &"#495057")
    })?;

    let mut custom_data: SceneFields = elements[frame]
        .get("customData")
        .map(|value| facet_json::from_str::<Option<SceneFields>>(value.as_str()))
        .transpose()?
        .flatten()
        .unwrap_or_default();
    let old_layout: SceneFields = custom_data
        .get("layout")
        .map(|value| facet_json::from_str::<Option<SceneFields>>(value.as_str()))
        .transpose()?
        .flatten()
        .unwrap_or_default();
    let old_groups: Vec<String> = old_layout
        .get("groups")
        .map(|value| facet_json::from_str(value.as_str()))
        .transpose()?
        .unwrap_or_default();
    let groups: Vec<_> = (0..records.len())
        .map(|slot| format!("ct-log-{output_id}-{slot}"))
        .collect();
    let mut active_ids = BTreeSet::new();
    for (slot, (record, group)) in records.iter().zip(&groups).enumerate() {
        let y = frame_bounds.y + top + slot as f64 * (row_height + GAP);
        let level_width = font_size * 4.0;
        let fields_width = width * 0.24;
        let message_width = width - level_width - fields_width - GAP * 2.0;
        let fields = record
            .fields
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("\n");
        let label = record.level.label().to_ascii_uppercase();
        for (part, content, x, part_width, color) in [
            (
                "level",
                label.as_str(),
                bounds.x,
                level_width,
                record.level.stroke_color(),
            ),
            (
                "message",
                record.message.as_str(),
                bounds.x + level_width + GAP,
                message_width,
                "#1e1e1e",
            ),
            (
                "fields",
                fields.as_str(),
                bounds.x + width - fields_width,
                fields_width,
                "#5c5f66",
            ),
        ] {
            let id = format!("{group}-{part}");
            active_ids.insert(id.clone());
            let mut desired = crate::interactive_demo::text(
                &id, content, x, y, part_width, row_height, font_size,
            )?;
            set(&mut desired, "frameId", &frame_id)?;
            set(&mut desired, "groupIds", &vec![group])?;
            set(&mut desired, "strokeColor", &color)?;
            set(&mut desired, "containerId", &Option::<String>::None)?;
            if let Some(existing) = elements
                .iter_mut()
                .find(|element| string(element, "id").as_deref() == Some(&id))
            {
                changed |= edit(existing, |element| {
                    // Only own presentation fields. Keep opaque editor data and
                    // native element identity while reusing a bounded set of slots.
                    for key in [
                        "text",
                        "originalText",
                        "x",
                        "y",
                        "width",
                        "height",
                        "fontSize",
                        "fontFamily",
                        "lineHeight",
                        "autoResize",
                        "strokeColor",
                        "frameId",
                        "groupIds",
                        "isDeleted",
                        "opacity",
                        "angle",
                    ] {
                        element.insert(key.into(), desired[key].clone());
                    }
                    // Text belongs to its row; copied editor bindings must not
                    // redirect layout into an unrelated user rectangle.
                    set(element, "containerId", &Option::<String>::None)
                })?;
            } else {
                elements.push(desired);
                changed = true;
            }
        }
    }
    // Retain tombstones for the editor's merging protocol, but recycle stable
    // slots on subsequent updates so long-running sessions don't grow the file.
    let mut owned_ids = BTreeSet::new();
    for group in old_groups
        .iter()
        .filter(|group| group.starts_with("ct-log-"))
        .cloned()
        .chain((0..MAX_ROWS).map(|slot| format!("ct-log-{output_id}-{slot}")))
    {
        for part in ["level", "message", "fields"] {
            owned_ids.insert(format!("{group}-{part}"));
        }
    }
    for element in elements.iter_mut() {
        if string(element, "id")
            .is_some_and(|id| owned_ids.contains(&id) && !active_ids.contains(&id))
        {
            changed |= edit(element, |element| set(element, "isDeleted", &true))?;
        }
    }
    let layout = ExcalidrawRowLayout {
        groups,
        top,
        bottom: BOTTOM_PADDING,
        gap: GAP,
    };
    let mut layout_fields: SceneFields = facet_json::from_str(&facet_json::to_string(&layout)?)?;
    set(&mut layout_fields, "type", &"rows")?;
    set(&mut custom_data, "layout", &layout_fields)?;
    changed |= edit(&mut elements[frame], |element| {
        set(element, "customData", &custom_data)
    })?;
    Ok(changed)
}
