use crate::ExcalidrawLogLevel;
use crate::ExcalidrawLogRecord;
use crate::excalidraw_control_bounds::ExcalidrawControlBounds;
use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::deleted;
use crate::interactive_scene_fields::edit;
use crate::interactive_scene_fields::log_output;
use crate::interactive_scene_fields::set;
use crate::interactive_scene_fields::string;
use crate::interactive_scene_fields::text;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet_json::RawJson;
use std::collections::BTreeMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Loss-preserving editor for the visible controls used by `ct excalidraw
/// interactive`. Unknown JSON properties and element kinds are retained as raw
/// JSON, including images and deleted-element tombstones. This deliberately does
/// not deserialize an arbitrary drawing into the exporter's smaller enum.
///
/// See the [Excalidraw scene format](https://docs.excalidraw.com/docs/codebase/json-schema).
#[derive(Debug, Clone)]
pub struct ExcalidrawInteractiveDocument {
    // Private because `elements` is separated from the root fields while editing.
    fields: SceneFields,
    elements: Vec<SceneFields>,
}

impl ExcalidrawInteractiveDocument {
    // Shared only with loss-preserving editors in this crate. Root fields stay
    // separate, and callers serialize through this document's boundary.
    pub(crate) fn elements(&self) -> &[SceneFields] {
        &self.elements
    }

    pub(crate) fn elements_mut(&mut self) -> &mut Vec<SceneFields> {
        &mut self.elements
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let mut fields: SceneFields = facet_json::from_str(json.trim_start_matches('\u{feff}'))
            .wrap_err("reading Excalidraw document object")?;
        if string(&fields, "type").as_deref() != Some("excalidraw") {
            bail!("expected an Excalidraw JSON scene (type = excalidraw)");
        }
        let elements = fields
            .remove("elements")
            .ok_or_else(|| eyre::eyre!("drawing has no elements array"))?;
        let elements =
            facet_json::from_str(elements.as_str()).wrap_err("reading Excalidraw elements")?;
        Ok(Self { fields, elements })
    }

    pub fn to_json(&self) -> Result<String> {
        let mut fields = self.fields.clone();
        set(&mut fields, "elements", &self.elements)?;
        facet_json::to_string_pretty(&fields).wrap_err("serializing edited Excalidraw drawing")
    }

    /// A local playground with visible question, reload, and log controls. No
    /// identity or decision semantics are hidden in element IDs or customData.
    pub fn demo() -> Result<Self> {
        let mut fields = SceneFields::new();
        set(&mut fields, "type", &"excalidraw")?;
        set(&mut fields, "version", &2)?;
        set(
            &mut fields,
            "source",
            &"https://github.com/AAFC-Cloud/Cloud-Terrastodon",
        )?;
        fields.insert(
            "appState".into(),
            RawJson::from_owned("{\"viewBackgroundColor\":\"#ffffff\",\"gridSize\":null}".into()),
        );
        fields.insert("files".into(), RawJson::from_owned("{}".into()));
        Ok(Self {
            fields,
            elements: crate::interactive_demo::elements()?,
        })
    }

    /// Each `#UserQuestion…` text label belongs to exactly one rectangle. The
    /// answer is its one other contained text element; ambiguous controls are
    /// omitted and explained by `diagnostics()`. Deleted elements are ignored.
    pub fn questions(&self) -> BTreeMap<String, String> {
        self.inspect_questions().0
    }

    /// `None` means incomplete or ambiguous controls, not an implicit default.
    pub fn selected_log_level(&self) -> Option<ExcalidrawLogLevel> {
        self.inspect_logs().ok().map(|(_, _, level)| level)
    }

    pub fn diagnostics(&self) -> Vec<String> {
        let mut diagnostics = self.inspect_questions().1;
        if let Err(message) = self.inspect_logs() {
            diagnostics.push(message);
        }
        let status_count = self.marker_indices("#CTLiveReload").len();
        if status_count > 1 {
            diagnostics.push("Multiple #CTLiveReload labels; status update is ambiguous".into());
        }
        diagnostics
    }

    /// Render already-filtered events as native level/message/fields text rows
    /// below `#Logs`. The editor measures and wraps intact text, then applies an
    /// optional row layout hint on the frame. Visible labels remain the control
    /// identity; the hint only governs presentation. Existing controls receive
    /// the same level palette as new demo drawings. The movable selector adopts
    /// the selected level's palette when CT publishes the filtered log rows,
    /// acknowledging the saved selection without changing its geometry.
    pub fn set_logs(&mut self, records: &[ExcalidrawLogRecord]) -> Result<bool> {
        let Ok((frame, output, level)) = self.inspect_logs() else {
            return Ok(false);
        };
        let frame_bounds = ExcalidrawControlBounds::read(&self.elements[frame]).unwrap();
        let labels: Vec<_> = self
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| !deleted(element) && !log_output(element))
            .filter(|(_, element)| {
                ExcalidrawControlBounds::read(element)
                    .is_some_and(|child| frame_bounds.contains(child))
            })
            .filter_map(|(index, element)| {
                text(element)
                    .as_deref()
                    .and_then(ExcalidrawLogLevel::from_label)
                    .map(|level| (index, level))
            })
            .collect();
        let selector = self
            .elements
            .iter()
            .enumerate()
            .find(|(_, element)| {
                !deleted(element)
                    && !log_output(element)
                    && text(element).as_deref() == Some("Log Level")
                    && ExcalidrawControlBounds::read(element)
                        .is_some_and(|child| frame_bounds.contains(child))
            })
            .and_then(|(label, _)| {
                self.containing_rectangle(label, None)
                    .ok()
                    .map(|rectangle| (rectangle, label))
            });
        let mut changed = false;
        for (label, target_level) in labels {
            if let Ok(target) =
                self.containing_rectangle(label, selector.map(|(rectangle, _)| rectangle))
            {
                changed |= edit(&mut self.elements[target], |element| {
                    set(element, "strokeColor", &target_level.stroke_color())?;
                    set(element, "backgroundColor", &target_level.background_color())?;
                    set(element, "fillStyle", &"solid")
                })?;
                changed |= edit(&mut self.elements[label], |element| {
                    set(element, "strokeColor", &target_level.stroke_color())
                })?;
            }
        }
        if let Some((selector, label)) = selector {
            changed |= edit(&mut self.elements[selector], |element| {
                set(element, "strokeColor", &level.stroke_color())?;
                set(element, "backgroundColor", &level.background_color())?;
                set(element, "fillStyle", &"solid")
            })?;
            changed |= edit(&mut self.elements[label], |element| {
                set(element, "strokeColor", &level.stroke_color())
            })?;
        }
        changed |=
            crate::interactive_logs::update(&mut self.elements, frame, output, level, records)?;
        Ok(changed)
    }

    /// Space is an explicit request to update the visible reload marker. For an
    /// existing drawing without one, append a marker below its current bounds;
    /// do not reposition existing elements or create the whole demo board.
    pub fn set_test_status(&mut self, message: &str) -> Result<bool> {
        let indices = self.marker_indices("#CTLiveReload");
        let content = format!("#CTLiveReload\n{message}");
        match indices.as_slice() {
            [index] => self.replace_text(*index, content),
            [] => {
                let bottom = self
                    .elements
                    .iter()
                    .filter(|element| !deleted(element))
                    .filter_map(ExcalidrawControlBounds::read)
                    .map(|bounds| bounds.y + bounds.height)
                    .fold(0.0, f64::max);
                let id = format!(
                    "ct-live-reload-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                );
                self.elements.push(crate::interactive_demo::text(
                    &id,
                    &content,
                    40.0,
                    bottom + 40.0,
                    650.0,
                    60.0,
                    18.0,
                )?);
                Ok(true)
            }
            _ => {
                bail!("multiple #CTLiveReload labels; remove the ambiguity before updating status");
            }
        }
    }

    fn replace_text(&mut self, index: usize, content: String) -> Result<bool> {
        if text(&self.elements[index]).as_deref() == Some(content.as_str()) {
            return Ok(false);
        }
        edit(&mut self.elements[index], |element| {
            set(element, "text", &content)?;
            set(element, "originalText", &content)
        })
    }

    fn marker_indices(&self, marker: &str) -> Vec<usize> {
        self.elements
            .iter()
            .enumerate()
            .filter(|(_, element)| !deleted(element) && !log_output(element))
            .filter(|(_, element)| {
                text(element)
                    .is_some_and(|value| value.lines().next().map(str::trim) == Some(marker))
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn containing_rectangle(
        &self,
        label: usize,
        exclude: Option<usize>,
    ) -> std::result::Result<usize, String> {
        let label_bounds = ExcalidrawControlBounds::read(&self.elements[label])
            .ok_or("Control has rotated or invalid geometry")?;
        let bound_id = string(&self.elements[label], "containerId");
        let candidates: Vec<_> = self
            .elements
            .iter()
            .enumerate()
            .filter(|(index, element)| {
                Some(*index) != exclude
                    && !deleted(element)
                    && string(element, "type").as_deref() == Some("rectangle")
            })
            .filter(|(_, element)| {
                bound_id
                    .as_ref()
                    .is_none_or(|id| string(element, "id").as_ref() == Some(id))
            })
            .filter(|(_, element)| {
                ExcalidrawControlBounds::read(element)
                    .is_some_and(|bounds| bounds.contains(label_bounds))
            })
            .map(|(index, _)| index)
            .collect();
        match candidates.as_slice() {
            [index] => Ok(*index),
            [] => Err("Control needs one unrotated enclosing rectangle".into()),
            _ => Err("Control is inside multiple rectangles; containment is ambiguous".into()),
        }
    }

    fn inspect_questions(&self) -> (BTreeMap<String, String>, Vec<String>) {
        let mut labels = BTreeMap::<String, Vec<usize>>::new();
        for (index, element) in self
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| !deleted(element) && !log_output(element))
        {
            if let Some(label) =
                text(element)
                    .map(|label| label.trim().to_owned())
                    .filter(|label| {
                        label.starts_with("#UserQuestion")
                            && !label.chars().any(char::is_whitespace)
                    })
            {
                labels.entry(label).or_default().push(index);
            }
        }
        let mut questions = BTreeMap::new();
        let mut diagnostics = Vec::new();
        for (label, indices) in labels {
            if indices.len() != 1 {
                diagnostics.push(format!("{label}: duplicate visible identifier"));
                continue;
            }
            let container = match self.containing_rectangle(indices[0], None) {
                Ok(index) => index,
                Err(message) => {
                    diagnostics.push(format!("{label}: {message}"));
                    continue;
                }
            };
            let bounds = ExcalidrawControlBounds::read(&self.elements[container]).unwrap();
            if self.elements.iter().enumerate().any(|(index, element)| {
                index != indices[0]
                    && !deleted(element)
                    && text(element).is_some()
                    && ExcalidrawControlBounds::read(element).is_none()
                    && ExcalidrawControlBounds::unrotated(element)
                        .is_some_and(|child| bounds.contains_center(child))
            }) {
                diagnostics.push(format!("{label}: rotated answer text is unsupported"));
                continue;
            }
            let answers: Vec<_> = self
                .elements
                .iter()
                .enumerate()
                .filter(|(index, element)| {
                    *index != indices[0] && !deleted(element) && !log_output(element)
                })
                .filter(|(_, element)| {
                    ExcalidrawControlBounds::read(element)
                        .is_some_and(|child| bounds.contains(child))
                })
                .filter_map(|(_, element)| text(element))
                .filter(|answer| !answer.trim_start().starts_with('#'))
                .collect();
            match answers.as_slice() {
                [answer] => {
                    questions.insert(label, answer.trim().to_owned());
                }
                [] => diagnostics.push(format!("{label}: no answer text inside its rectangle")),
                _ => diagnostics.push(format!(
                    "{label}: multiple answer texts inside its rectangle"
                )),
            }
        }
        (questions, diagnostics)
    }

    fn inspect_logs(&self) -> std::result::Result<(usize, usize, ExcalidrawLogLevel), String> {
        let frames: Vec<_> = self
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                !deleted(element)
                    && string(element, "type").as_deref() == Some("frame")
                    && string(element, "name").as_deref() == Some("Logs")
            })
            .map(|(index, _)| index)
            .collect();
        let [frame] = frames.as_slice() else {
            return Err("Logs needs exactly one native frame named Logs".into());
        };
        let bounds = ExcalidrawControlBounds::read(&self.elements[*frame])
            .ok_or("Logs frame has rotated or invalid geometry")?;
        let inside = |index: usize| {
            ExcalidrawControlBounds::read(&self.elements[index])
                .is_some_and(|child| bounds.contains(child))
        };
        let selectors: Vec<_> = self
            .elements
            .iter()
            .enumerate()
            .filter(|(index, element)| {
                !deleted(element)
                    && !log_output(element)
                    && text(element).as_deref() == Some("Log Level")
                    && inside(*index)
            })
            .map(|(index, _)| index)
            .collect();
        let [selector_label] = selectors.as_slice() else {
            return Err("Logs needs one visible Log Level selector".into());
        };
        let selector = self.containing_rectangle(*selector_label, None)?;
        if !inside(selector) {
            return Err("Log Level selector must stay inside Logs".into());
        }
        let selector_bounds = ExcalidrawControlBounds::read(&self.elements[selector]).unwrap();
        let mut matches = Vec::new();
        for (index, element) in
            self.elements.iter().enumerate().filter(|(index, element)| {
                !deleted(element) && !log_output(element) && inside(*index)
            })
        {
            let Some(level) = text(element)
                .as_deref()
                .and_then(ExcalidrawLogLevel::from_label)
            else {
                continue;
            };
            let target = self.containing_rectangle(index, Some(selector))?;
            if inside(target)
                && ExcalidrawControlBounds::read(&self.elements[target])
                    .unwrap()
                    .contains_center(selector_bounds)
            {
                matches.push(level);
            }
        }
        let [level] = matches.as_slice() else {
            return Err("Place the Log Level selector center inside exactly one Debug/Info/Warn/Error/Off rectangle".into());
        };
        let outputs: Vec<_> = self
            .marker_indices("#Logs")
            .into_iter()
            .filter(|index| inside(*index))
            .collect();
        let [output] = outputs.as_slice() else {
            return Err("Logs needs exactly one unrotated #Logs text body inside its frame".into());
        };
        Ok((*frame, *output, *level))
    }
}

#[cfg(test)]
#[path = "excalidraw_interactive_document_tests.rs"]
mod tests;
