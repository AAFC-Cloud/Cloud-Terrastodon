use crate::ExcalidrawCanvasAction;
use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::ExcalidrawInteractiveDocument;
use crate::excalidraw_control_bounds::ExcalidrawControlBounds;
use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::deleted;
use crate::interactive_scene_fields::edit;
use crate::interactive_scene_fields::set;
use crate::interactive_scene_fields::string;
use eyre::Result;
use eyre::bail;

const STROKE: &str = "#6741d9";
const FILL: &str = "#d0bfff";
const ERROR: &str = "#c92a2a";
const GAP: f64 = 16.0;

impl ExcalidrawInteractiveDocument {
    /// Add a slide control below the frame's existing contents. A purple square
    /// already in the frame is treated as a deliberate control, including one
    /// whose caption needs repair, instead of adding a competing default.
    pub fn ensure_action_control(
        &mut self,
        frame_name: &str,
        action: ExcalidrawCanvasAction,
    ) -> Result<bool> {
        let frame = frame(self.elements(), frame_name)?;
        let frame_bounds = bounds(&self.elements()[frame])?;
        let frame_id = id(&self.elements()[frame])?;
        if !gates(self.elements(), &frame_id, frame_bounds).is_empty() {
            return Ok(false);
        }
        let bottom = self
            .elements()
            .iter()
            .filter(|element| !deleted(element))
            .filter(|element| string(element, "type").as_deref() != Some("frame"))
            .filter_map(ExcalidrawControlBounds::read)
            .filter(|child| frame_bounds.contains_center(*child))
            .map(|child| child.y + child.height)
            .fold(frame_bounds.y + 20.0, f64::max);
        let gate_id = fresh_id(self.elements(), &format!("ct-action-{frame_id}"));
        let gate = ExcalidrawControlBounds {
            x: frame_bounds.x + 70.0,
            y: bottom + 24.0,
            width: 42.0,
            height: 42.0,
        };
        self.elements_mut()
            .push(rectangle(&gate_id, &frame_id, gate)?);
        self.elements_mut().push(caption(
            &format!("{gate_id}-label"),
            &frame_id,
            action.label(),
            gate.x + gate.width + GAP,
            gate.y + 8.0,
        )?);
        self.elements_mut().push(handle(&gate_id, &frame_id, gate)?);
        grow_frame(self.elements_mut(), frame, gate.x + 260.0, gate.y + 64.0)?;
        Ok(true)
    }

    /// Repair controls and acknowledge fully nested handles by moving them back
    /// beside their gates. The returned actions describe this edit only: callers
    /// must successfully persist it before invoking an action. A second pass on
    /// the acknowledged scene returns no action, including after reopening it.
    ///
    /// Only unrotated purple squares, 32–80 canvas units across, inside the named
    /// frame participate. Captions must be immediately to the right. Other
    /// rectangles, annotations and frames remain untouched. Errors are visible
    /// red text, and only this containing frame grows to accommodate repairs.
    pub fn reconcile_action_controls(
        &mut self,
        frame_name: &str,
    ) -> Result<(bool, Vec<ExcalidrawCanvasAction>)> {
        let frame = frame(self.elements(), frame_name)?;
        let frame_bounds = bounds(&self.elements()[frame])?;
        let frame_id = id(&self.elements()[frame])?;
        let gates = gates(self.elements(), &frame_id, frame_bounds);
        let mut changed = false;
        let mut actions = Vec::new();
        for gate_index in gates {
            let gate = bounds(&self.elements()[gate_index])?;
            let gate_id = id(&self.elements()[gate_index])?;
            let labels: Vec<_> = self
                .elements()
                .iter()
                .enumerate()
                .filter(|(_, element)| !deleted(element))
                .filter(|(_, element)| string(element, "type").as_deref() == Some("text"))
                .filter(|(_, element)| belongs_to_frame(element, &frame_id, frame_bounds))
                .filter(|(_, element)| {
                    ExcalidrawControlBounds::read(element).is_some_and(|label| {
                        let right = gate.x + gate.width;
                        label.x >= right
                            && label.x - right <= 80.0
                            && (label.y + label.height / 2.0 - gate.y - gate.height / 2.0).abs()
                                <= gate.height / 2.0 + 8.0
                    })
                })
                .map(|(index, _)| index)
                .collect();
            let (action, mut error) = match labels.as_slice() {
                [label] => {
                    let label = string(&self.elements()[*label], "originalText")
                        .or_else(|| string(&self.elements()[*label], "text"))
                        .unwrap_or_default();
                    match ExcalidrawCanvasAction::from_label(&label) {
                        Some(action) => (Some(action), None),
                        None => (None, Some("Unknown action. Use Add Target.")),
                    }
                }
                [] => {
                    let label = caption(
                        &fresh_id(self.elements(), &format!("{gate_id}-label")),
                        &frame_id,
                        "Unknown Action",
                        gate.x + gate.width + GAP,
                        gate.y + 8.0,
                    )?;
                    self.elements_mut().push(label);
                    changed = true;
                    (None, Some("Unknown action. Use Add Target."))
                }
                _ => (
                    None,
                    Some("Ambiguous action: keep one caption to the right."),
                ),
            };
            let handles: Vec<_> = self
                .elements()
                .iter()
                .enumerate()
                .filter(|(_, element)| !deleted(element) && purple(element))
                .filter(|(_, element)| belongs_to_frame(element, &frame_id, frame_bounds))
                .filter_map(|(index, element)| {
                    let child = ExcalidrawControlBounds::read(element)?;
                    (small_handle(gate, child)
                        && (child.x + child.width / 2.0 - gate.x - gate.width / 2.0).abs() <= 110.0
                        && (child.y + child.height / 2.0 - gate.y - gate.height / 2.0).abs()
                            <= 80.0)
                        .then_some((index, child))
                })
                .collect();
            match handles.as_slice() {
                [(handle, child)] if gate.contains(*child) => {
                    // Reset even malformed controls; fixing a caption later must
                    // not replay an old gesture as an unexpected new action.
                    changed |= reset_handle(&mut self.elements_mut()[*handle], gate)?;
                    if let Some(action) = action {
                        actions.push(action);
                    }
                }
                [_] => {}
                [] => {
                    let handle_id = format!("{gate_id}-handle");
                    if let Some(index) = self
                        .elements()
                        .iter()
                        .position(|element| string(element, "id").as_deref() == Some(&handle_id))
                    {
                        changed |= edit(&mut self.elements_mut()[index], |element| {
                            set(element, "isDeleted", &false)?;
                            set(element, "type", &"rectangle")?;
                            set(element, "frameId", &frame_id)?;
                            set(element, "angle", &0.0)?;
                            set(element, "strokeColor", &STROKE)?;
                            set(element, "backgroundColor", &FILL)?;
                            set(element, "fillStyle", &"solid")?;
                            set(element, "width", &20.0)?;
                            set(element, "height", &20.0)
                        })?;
                        changed |= reset_handle(&mut self.elements_mut()[index], gate)?;
                    } else {
                        self.elements_mut().push(handle(&gate_id, &frame_id, gate)?);
                        changed = true;
                    }
                }
                _ => {
                    error = Some("Ambiguous slider: leave one small purple handle nearby.");
                }
            }
            changed |= update_error(self.elements_mut(), frame, gate_index, error)?;
            changed |= grow_frame(
                self.elements_mut(),
                frame,
                gate.x + gate.width + 240.0,
                gate.y + gate.height + 20.0,
            )?;
        }
        Ok((changed, actions))
    }
}

fn frame(elements: &[SceneFields], name: &str) -> Result<usize> {
    let frames: Vec<_> = elements
        .iter()
        .enumerate()
        .filter(|(_, element)| !deleted(element))
        .filter(|(_, element)| {
            string(element, "type").as_deref() == Some("frame")
                && string(element, "name").as_deref() == Some(name)
        })
        .map(|(index, _)| index)
        .collect();
    match frames.as_slice() {
        [frame] => {
            bounds(&elements[*frame])?;
            Ok(*frame)
        }
        _ => {
            bail!("expected one unrotated {name} frame for action controls");
        }
    }
}

fn bounds(element: &SceneFields) -> Result<ExcalidrawControlBounds> {
    ExcalidrawControlBounds::read(element)
        .ok_or_else(|| eyre::eyre!("action controls require finite, unrotated geometry"))
}

fn id(element: &SceneFields) -> Result<String> {
    string(element, "id").ok_or_else(|| eyre::eyre!("action control has no element ID"))
}

fn purple(element: &SceneFields) -> bool {
    string(element, "type").as_deref() == Some("rectangle")
        && string(element, "strokeColor").is_some_and(|color| color.eq_ignore_ascii_case(STROKE))
        && string(element, "backgroundColor").is_some_and(|color| color.eq_ignore_ascii_case(FILL))
}

fn gates(elements: &[SceneFields], frame_id: &str, frame: ExcalidrawControlBounds) -> Vec<usize> {
    elements
        .iter()
        .enumerate()
        .filter(|(_, element)| !deleted(element) && purple(element))
        .filter(|(_, element)| belongs_to_frame(element, frame_id, frame))
        .filter_map(|(index, element)| {
            let gate = ExcalidrawControlBounds::read(element)?;
            (frame.contains(gate)
                && (32.0..=80.0).contains(&gate.width)
                && (32.0..=80.0).contains(&gate.height)
                && (0.75..=1.34).contains(&(gate.width / gate.height)))
            .then_some(index)
        })
        .collect()
}

fn small_handle(gate: ExcalidrawControlBounds, handle: ExcalidrawControlBounds) -> bool {
    handle.width >= 6.0
        && handle.height >= 6.0
        && handle.width <= gate.width * 0.7
        && handle.height <= gate.height * 0.7
}

fn belongs_to_frame(element: &SceneFields, frame_id: &str, frame: ExcalidrawControlBounds) -> bool {
    match string(element, "frameId") {
        Some(owner) => owner == frame_id,
        None => {
            ExcalidrawControlBounds::read(element).is_some_and(|child| frame.contains_center(child))
        }
    }
}

fn fresh_id(elements: &[SceneFields], prefix: &str) -> String {
    (0..)
        .map(|index| format!("{prefix}-{index}"))
        .find(|id| {
            !elements
                .iter()
                .any(|element| string(element, "id").as_deref() == Some(id))
        })
        .unwrap()
}

fn rectangle(id: &str, frame: &str, bounds: ExcalidrawControlBounds) -> Result<SceneFields> {
    let base = ExcalidrawElementBase::new(
        ExcalidrawElementId(id.into()),
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
    );
    let mut element = facet_json::from_str(&facet_json::to_string(&base)?)?;
    set(&mut element, "type", &"rectangle")?;
    set(&mut element, "frameId", &frame)?;
    set(&mut element, "strokeColor", &STROKE)?;
    set(&mut element, "backgroundColor", &FILL)?;
    set(&mut element, "fillStyle", &"solid")?;
    Ok(element)
}

fn caption(id: &str, frame: &str, label: &str, x: f64, y: f64) -> Result<SceneFields> {
    let mut element = crate::interactive_demo::text(id, label, x, y, 220.0, 25.0, 20.0)?;
    set(&mut element, "fontFamily", &5)?;
    set(&mut element, "frameId", &frame)?;
    Ok(element)
}

fn handle(id: &str, frame: &str, gate: ExcalidrawControlBounds) -> Result<SceneFields> {
    rectangle(
        &format!("{id}-handle"),
        frame,
        ExcalidrawControlBounds {
            x: gate.x - 40.0,
            y: gate.y + (gate.height - 20.0) / 2.0,
            width: 20.0,
            height: 20.0,
        },
    )
}

fn reset_handle(element: &mut SceneFields, gate: ExcalidrawControlBounds) -> Result<bool> {
    let handle = bounds(element)?;
    edit(element, |element| {
        set(element, "x", &(gate.x - GAP - handle.width))?;
        set(
            element,
            "y",
            &(gate.y + (gate.height - handle.height) / 2.0),
        )
    })
}

fn grow_frame(elements: &mut [SceneFields], frame: usize, right: f64, bottom: f64) -> Result<bool> {
    let current = bounds(&elements[frame])?;
    edit(&mut elements[frame], |element| {
        set(element, "width", &current.width.max(right - current.x))?;
        set(element, "height", &current.height.max(bottom - current.y))
    })
}

fn update_error(
    elements: &mut Vec<SceneFields>,
    frame: usize,
    gate: usize,
    error: Option<&str>,
) -> Result<bool> {
    let error_id = format!("{}-error", id(&elements[gate])?);
    let existing = elements
        .iter()
        .position(|element| string(element, "id").as_deref() == Some(&error_id));
    let Some(error) = error else {
        return match existing {
            Some(index) if !deleted(&elements[index]) => edit(&mut elements[index], |element| {
                set(element, "isDeleted", &true)
            }),
            _ => Ok(false),
        };
    };
    let gate = bounds(&elements[gate])?;
    let frame_bounds = bounds(&elements[frame])?;
    let frame_id = id(&elements[frame])?;
    let mut y = gate.y + gate.height + 12.0;
    // Allocate the error below this control without shifting hand-positioned
    // content. Skip occupied rows; growing the parent makes the result visible.
    loop {
        let bottom = elements
            .iter()
            .enumerate()
            .filter(|(index, element)| Some(*index) != existing && !deleted(element))
            .filter(|(_, element)| string(element, "type").as_deref() != Some("frame"))
            .filter(|(_, element)| {
                string(element, "frameId").as_deref() == Some(frame_id.as_str())
                    || ExcalidrawControlBounds::read(element)
                        .is_some_and(|child| frame_bounds.contains_center(child))
            })
            .filter_map(|(_, element)| ExcalidrawControlBounds::read(element))
            .filter(|child| {
                child.x < gate.x + 380.0
                    && child.x + child.width > gate.x
                    && child.y < y + 40.0
                    && child.y + child.height > y
            })
            .map(|child| child.y + child.height + 12.0)
            .max_by(f64::total_cmp);
        match bottom {
            Some(bottom) => y = bottom,
            None => break,
        }
    }
    let mut changed = match existing {
        Some(index) => edit(&mut elements[index], |element| {
            set(element, "isDeleted", &false)?;
            set(element, "text", &error)?;
            set(element, "originalText", &error)?;
            set(element, "x", &gate.x)?;
            set(element, "y", &y)?;
            set(element, "strokeColor", &ERROR)
        })?,
        None => {
            let mut message =
                crate::interactive_demo::text(&error_id, error, gate.x, y, 380.0, 40.0, 16.0)?;
            set(&mut message, "frameId", &frame_id)?;
            set(&mut message, "strokeColor", &ERROR)?;
            elements.push(message);
            true
        }
    };
    changed |= grow_frame(elements, frame, gate.x + 400.0, y + 60.0)?;
    Ok(changed)
}

#[cfg(test)]
#[path = "excalidraw_action_control_tests.rs"]
mod tests;
