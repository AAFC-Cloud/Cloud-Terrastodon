use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::excalidraw_control_bounds::ExcalidrawControlBounds;
use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::deleted;
use crate::interactive_scene_fields::edit;
use crate::interactive_scene_fields::number;
use crate::interactive_scene_fields::set;
use crate::interactive_scene_fields::string;
use eyre::Result;
use eyre::bail;

const STROKE: &str = "#7048e8";
const THICKNESS: f64 = 14.0;
const MIN_LENGTH: f64 = 28.0;

/// A native line and draggable rectangle controlling a window into a collection.
/// The visible `#Scroll` caption records the published range and query, so a
/// reopened drawing retains its offset and a resized frame retains the same
/// first item. Element IDs identify presentation updates only; the caption,
/// purple controls and frame geometry identify the control's meaning.
///
/// Its private indices and geometry belong to one reconciliation pass. Call
/// `prepare`, compute the number of cards fitting `content_bounds`, then `page`.
pub(crate) struct ExcalidrawScrollbar {
    track: usize,
    thumb: usize,
    status: usize,
    content: ExcalidrawControlBounds,
    vertical: bool,
    start: f64,
    cross: f64,
    length: f64,
    previous: Option<(usize, usize, usize, String)>,
    drag: Option<f64>,
    changed: bool,
}

impl ExcalidrawScrollbar {
    pub fn prepare(
        elements: &mut Vec<SceneFields>,
        frame: usize,
        header_height: f64,
    ) -> Result<Self> {
        let frame_bounds = ExcalidrawControlBounds::read(&elements[frame])
            .ok_or_else(|| eyre::eyre!("scrollbar frame needs unrotated finite geometry"))?;
        let frame_id = string(&elements[frame], "id")
            .ok_or_else(|| eyre::eyre!("scrollbar frame needs an identifier"))?;
        let content = ExcalidrawControlBounds {
            x: frame_bounds.x + 20.0,
            y: frame_bounds.y + header_height,
            width: (frame_bounds.width - 40.0).max(0.0),
            height: (frame_bounds.height - header_height - 20.0).max(0.0),
        };
        let mut changed = false;
        let status = find_one(elements, |element| {
            belongs(element, &frame_id, frame_bounds)
                && original_text(element).is_some_and(|text| text.starts_with("#Scroll "))
        })?;
        let previous = status.and_then(|index| parse_status(&elements[index]));
        let status = if let Some(status) = status {
            status
        } else {
            let id = fresh_id(elements, "ct-scroll-status")?;
            let mut label = crate::interactive_demo::text(
                &id,
                "#Scroll 0–0 of 0 (0 slots)\nQuery: ",
                frame_bounds.x + 20.0,
                frame_bounds.y + header_height - 36.0,
                (frame_bounds.width - 40.0).max(1.0),
                30.0,
                12.0,
            )?;
            set(&mut label, "frameId", &frame_id)?;
            elements.push(label);
            changed = true;
            elements.len() - 1
        };
        let track = find_component(elements, &frame_id, frame_bounds, "line", "track")?;
        let old_track = track
            .map(|index| track_geometry(&elements[index]))
            .transpose()?;
        let vertical = old_track.is_none_or(|(vertical, _, _, _)| vertical);
        let positive_edge = old_track.is_none_or(|(_, _, cross, _)| {
            if vertical {
                cross >= frame_bounds.x + frame_bounds.width / 2.0
            } else {
                cross >= frame_bounds.y + frame_bounds.height / 2.0
            }
        });
        let (start, cross, length) = if vertical {
            (
                content.y,
                if positive_edge {
                    frame_bounds.x + frame_bounds.width - 10.0
                } else {
                    frame_bounds.x + 10.0
                },
                content.height,
            )
        } else {
            (
                content.x,
                if positive_edge {
                    frame_bounds.y + frame_bounds.height - 10.0
                } else {
                    // Keep the top track in the outer frame margin, above the
                    // query and status, rather than crossing the status text.
                    frame_bounds.y + 10.0
                },
                content.width,
            )
        };
        let track = if let Some(track) = track {
            track
        } else {
            let mut track = component(elements, &frame_id, "line", "track")?;
            set(&mut track, "points", &vec![[0.0, 0.0], [0.0, length]])?;
            elements.push(track);
            changed = true;
            elements.len() - 1
        };
        let thumb = find_component(elements, &frame_id, frame_bounds, "rectangle", "thumb")?;
        let drag = match (thumb, old_track, &previous) {
            (
                Some(thumb),
                Some((old_vertical, old_start, old_cross, old_length)),
                Some((offset, total, capacity, _)),
            ) => {
                let geometry =
                    ExcalidrawControlBounds::read(&elements[thumb]).ok_or_else(|| {
                        eyre::eyre!("scrollbar thumb needs unrotated finite geometry")
                    })?;
                let old_size = thumb_length(old_length, *total, *capacity);
                let old_travel = (old_length - old_size).max(0.0);
                let old_max = total.saturating_sub(*capacity);
                let expected = old_start + fraction(*offset, old_max) * old_travel;
                let (actual, actual_cross) = if old_vertical {
                    (geometry.y, geometry.x + geometry.width / 2.0)
                } else {
                    (geometry.x, geometry.y + geometry.height / 2.0)
                };
                // A frame resize leaves both old track and thumb untouched. Use
                // the old track for this comparison before snapping it to the
                // resized frame, preserving the exact published item offset.
                let reoriented = (geometry.height - geometry.width).abs() > 0.01
                    && old_vertical != (geometry.height > geometry.width);
                if reoriented
                    || ((actual - expected).abs() <= 0.01
                        && (actual_cross - old_cross).abs() <= 0.01)
                {
                    // Moving the line to another edge may change its axis.
                    // The untouched thumb still has its previous orientation;
                    // this is a layout edit, not a scroll to its other axis.
                    // A manually resized thumb with the wrong proportions is
                    // treated the same way: repair it at the published offset.
                    None
                } else if old_travel > 0.0 {
                    Some(((actual - old_start) / old_travel).clamp(0.0, 1.0))
                } else {
                    Some(0.0)
                }
            }
            _ => None,
        };
        let thumb = if let Some(thumb) = thumb {
            thumb
        } else {
            elements.push(component(elements, &frame_id, "rectangle", "thumb")?);
            changed = true;
            elements.len() - 1
        };
        changed |= edit(&mut elements[track], |element| {
            set_number(element, "x", if vertical { cross } else { start })?;
            set_number(element, "y", if vertical { start } else { cross })?;
            set_number(element, "width", if vertical { 0.0 } else { length })?;
            set_number(element, "height", if vertical { length } else { 0.0 })?;
            set_number(element, "angle", 0.0)?;
            let points = vec![
                [0.0, 0.0],
                if vertical {
                    [0.0, length]
                } else {
                    [length, 0.0]
                },
            ];
            if element
                .get("points")
                .and_then(|value| facet_json::from_str::<Vec<[f64; 2]>>(value.as_str()).ok())
                .as_ref()
                != Some(&points)
            {
                set(element, "points", &points)?;
            }
            set(element, "frameId", &frame_id)
        })?;
        changed |= edit(&mut elements[status], |element| {
            set_number(element, "x", frame_bounds.x + 20.0)?;
            set_number(element, "y", frame_bounds.y + header_height - 36.0)?;
            set_number(element, "width", (frame_bounds.width - 40.0).max(1.0))?;
            set(element, "frameId", &frame_id)
        })?;
        Ok(Self {
            track,
            thumb,
            status,
            content,
            vertical,
            start,
            cross,
            length,
            previous,
            drag,
            changed,
        })
    }

    /// Exact available card area; callers add their inter-card gaps, not another
    /// outer margin. The track uses the frame's existing margin on any edge.
    pub fn content_bounds(&self) -> ExcalidrawControlBounds {
        self.content
    }

    pub fn element_indices(&self) -> [usize; 3] {
        [self.track, self.thumb, self.status]
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Interpret a saved thumb movement, clamp to a valid final page, and write
    /// the exact acknowledged position. Query edits reset to the first result;
    /// capacity/count changes clamp the previous first item instead of jumping
    /// to a new proportional position. Zero slots and empty results stay at zero.
    pub fn page(
        &mut self,
        elements: &mut [SceneFields],
        query: &str,
        total: usize,
        capacity: usize,
    ) -> Result<usize> {
        let maximum = total.saturating_sub(capacity);
        let offset = if capacity == 0 {
            0
        } else if let Some((previous, _, _, old_query)) = &self.previous {
            if old_query != query {
                0
            } else if let Some(fraction) = self.drag {
                (fraction * maximum as f64).round() as usize
            } else {
                (*previous).min(maximum)
            }
        } else {
            0
        };
        let size = thumb_length(self.length, total, capacity);
        let position = self.start + fraction(offset, maximum) * (self.length - size).max(0.0);
        let frame_id = string(&elements[self.track], "frameId");
        self.changed |= edit(&mut elements[self.thumb], |element| {
            set_number(
                element,
                "x",
                if self.vertical {
                    self.cross - THICKNESS / 2.0
                } else {
                    position
                },
            )?;
            set_number(
                element,
                "y",
                if self.vertical {
                    position
                } else {
                    self.cross - THICKNESS / 2.0
                },
            )?;
            set_number(
                element,
                "width",
                if self.vertical { THICKNESS } else { size },
            )?;
            set_number(
                element,
                "height",
                if self.vertical { size } else { THICKNESS },
            )?;
            set_number(element, "angle", 0.0)?;
            set(element, "frameId", &frame_id)
        })?;
        let shown = capacity.min(total);
        let status = format!(
            "#Scroll {}–{} of {total} ({capacity} slots)\nQuery: {query}",
            if shown == 0 { 0 } else { offset + 1 },
            offset + shown,
        );
        self.changed |= edit(&mut elements[self.status], |element| {
            if original_text(element).as_deref() != Some(&status) {
                set(element, "text", &status)?;
                set(element, "originalText", &status)?;
            }
            Ok(())
        })?;
        Ok(offset)
    }
}

fn thumb_length(length: f64, total: usize, capacity: usize) -> f64 {
    if total == 0 || capacity >= total {
        length
    } else {
        (length * capacity as f64 / total as f64)
            .max(MIN_LENGTH)
            .min(length)
    }
}

fn fraction(offset: usize, maximum: usize) -> f64 {
    if maximum == 0 {
        0.0
    } else {
        offset.min(maximum) as f64 / maximum as f64
    }
}

fn parse_status(element: &SceneFields) -> Option<(usize, usize, usize, String)> {
    let text = original_text(element)?;
    let (range, query) = text.strip_prefix("#Scroll ")?.split_once("\nQuery: ")?;
    let (first, rest) = range.split_once('–')?;
    let (_, rest) = rest.split_once(" of ")?;
    let (total, slots) = rest.split_once(" (")?;
    Some((
        first.parse::<usize>().ok()?.saturating_sub(1),
        total.parse().ok()?,
        slots.strip_suffix(" slots)")?.parse().ok()?,
        query.into(),
    ))
}

fn track_geometry(element: &SceneFields) -> Result<(bool, f64, f64, f64)> {
    let points: Vec<[f64; 2]> = element
        .get("points")
        .map(|value| facet_json::from_str(value.as_str()))
        .transpose()?
        .ok_or_else(|| eyre::eyre!("scrollbar track needs two endpoints"))?;
    let [first, last] = points.as_slice() else {
        bail!("scrollbar track needs exactly two endpoints");
    };
    let x = number(element, "x").unwrap_or(0.0);
    let y = number(element, "y").unwrap_or(0.0);
    let width = number(element, "width").unwrap_or(0.0);
    let height = number(element, "height").unwrap_or(0.0);
    let angle = number(element, "angle").unwrap_or(0.0);
    let rotate = |point: &[f64; 2]| {
        let dx = point[0] - width / 2.0;
        let dy = point[1] - height / 2.0;
        [
            x + width / 2.0 + dx * angle.cos() - dy * angle.sin(),
            y + height / 2.0 + dx * angle.sin() + dy * angle.cos(),
        ]
    };
    let first = rotate(first);
    let last = rotate(last);
    if !first
        .iter()
        .chain(last.iter())
        .all(|value| value.is_finite())
    {
        bail!("scrollbar track endpoints must be finite");
    }
    let vertical = (last[1] - first[1]).abs() >= (last[0] - first[0]).abs();
    Ok(if vertical {
        (
            true,
            first[1].min(last[1]),
            (first[0] + last[0]) / 2.0,
            (last[1] - first[1]).abs(),
        )
    } else {
        (
            false,
            first[0].min(last[0]),
            (first[1] + last[1]) / 2.0,
            (last[0] - first[0]).abs(),
        )
    })
}

fn find_one(
    elements: &[SceneFields],
    predicate: impl Fn(&SceneFields) -> bool,
) -> Result<Option<usize>> {
    let mut matches = elements
        .iter()
        .enumerate()
        .filter(|(_, element)| !deleted(element) && predicate(element));
    let first = matches.next().map(|(index, _)| index);
    if matches.next().is_some() {
        bail!(
            "ambiguous scrollbar: keep one #Scroll caption, purple line and purple thumb in each frame"
        );
    }
    Ok(first)
}

fn find_component(
    elements: &[SceneFields],
    frame_id: &str,
    frame: ExcalidrawControlBounds,
    kind: &str,
    role: &str,
) -> Result<Option<usize>> {
    let ownership = format!("{frame_id}-scroll-{role}");
    find_one(elements, |element| {
        string(element, "type").as_deref() == Some(kind)
            && string(element, "strokeColor").as_deref() == Some(STROKE)
            && (belongs(element, frame_id, frame)
                || element
                    .get("groupIds")
                    .and_then(|value| facet_json::from_str::<Vec<String>>(value.as_str()).ok())
                    .is_some_and(|groups| groups.contains(&ownership)))
    })
}

fn belongs(element: &SceneFields, frame_id: &str, frame: ExcalidrawControlBounds) -> bool {
    string(element, "frameId").as_deref() == Some(frame_id)
        || ExcalidrawControlBounds::unrotated(element)
            .is_some_and(|bounds| frame.contains_center(bounds))
}

fn component(
    elements: &[SceneFields],
    frame_id: &str,
    kind: &str,
    role: &str,
) -> Result<SceneFields> {
    let base = ExcalidrawElementBase::new(
        ExcalidrawElementId(fresh_id(elements, &format!("ct-scroll-{role}"))?),
        0.0,
        0.0,
        0.0,
        0.0,
    );
    let mut element: SceneFields = facet_json::from_str(&facet_json::to_string(&base)?)?;
    set(&mut element, "type", &kind)?;
    set(&mut element, "frameId", &frame_id)?;
    // Separate one-element groups retain repair ownership if a control is
    // dragged completely outside its frame; track and thumb remain independent.
    set(
        &mut element,
        "groupIds",
        &vec![format!("{frame_id}-scroll-{role}")],
    )?;
    set(&mut element, "strokeColor", &STROKE)?;
    set(
        &mut element,
        "backgroundColor",
        &if kind == "rectangle" {
            "#d0bfff"
        } else {
            "transparent"
        },
    )?;
    set(&mut element, "strokeWidth", &2.0)?;
    Ok(element)
}

fn original_text(element: &SceneFields) -> Option<String> {
    (string(element, "type").as_deref() == Some("text"))
        .then(|| string(element, "originalText").or_else(|| string(element, "text")))
        .flatten()
}

fn fresh_id(elements: &[SceneFields], prefix: &str) -> Result<String> {
    (0..=elements.len())
        .map(|index| format!("{prefix}-{index}"))
        .find(|id| {
            elements
                .iter()
                .all(|element| string(element, "id").as_deref() != Some(id))
        })
        .ok_or_else(|| eyre::eyre!("could not allocate an unused scrollbar element identifier"))
}

fn set_number(element: &mut SceneFields, name: &str, value: f64) -> Result<()> {
    if number(element, name).is_some_and(|previous| (previous - value).abs() < 0.000_001) {
        Ok(())
    } else {
        set(element, name, &value)
    }
}

#[cfg(test)]
#[path = "excalidraw_scrollbar_tests.rs"]
mod tests;
