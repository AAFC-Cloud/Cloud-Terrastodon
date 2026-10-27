use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::ExcalidrawInteractiveDocument;
use crate::ExcalidrawScene;
use crate::excalidraw_control_bounds::ExcalidrawControlBounds;
use crate::excalidraw_scrollbar::ExcalidrawScrollbar;
use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::deleted;
use crate::interactive_scene_fields::edit;
use crate::interactive_scene_fields::number;
use crate::interactive_scene_fields::set;
use crate::interactive_scene_fields::string;
use cloud_terrastodon_registry::DiscoveryPlan;
use eyre::Result;
use eyre::bail;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const POOL: &str = "Target search";
const LEGACY_POOL: &str = "Target pool";
const SELECTED: &str = "Selected targets";
const GENERATED: &str = "Generated plan";
const CARD_WIDTH: f64 = 390.0;
const CARD_HEIGHT: f64 = 85.0;
const GAP: f64 = 20.0;
const HEADER_HEIGHT: f64 = 150.0;
const SEARCH_HEIGHT: f64 = HEADER_HEIGHT + 5.0 * (CARD_HEIGHT + GAP);

/// A file-backed planning interface whose inputs are visible target labels and
/// their positions in native Excalidraw frames. Copy/paste may change element
/// identifiers without changing a target's meaning. Generated output uses IDs
/// only to update its own presentation while retaining unrelated scene data.
///
/// The board edits the [Excalidraw scene format](https://docs.excalidraw.com/docs/codebase/json-schema)
/// through the same loss-preserving boundary as the interactive playground.
#[derive(Debug, Clone)]
pub struct ExcalidrawPlanningBoard {
    // Own the preservation boundary; mutation of raw fields is crate-internal.
    document: ExcalidrawInteractiveDocument,
}

impl ExcalidrawPlanningBoard {
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(Self {
            document: ExcalidrawInteractiveDocument::from_json(json)?,
        })
    }

    pub fn to_json(&self) -> Result<String> {
        self.document.to_json()
    }

    pub fn document(&self) -> &ExcalidrawInteractiveDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut ExcalidrawInteractiveDocument {
        &mut self.document
    }

    /// Reopen a diagram without treating the canvas as an input interface.
    /// Retire generated controls from older planning boards, retaining unrelated
    /// annotations and detaching them from the retired frames. Terminal state
    /// now owns target selection; the canvas only owns diagram presentation.
    pub fn initialize_diagram(current_json: Option<&str>) -> Result<Self> {
        let mut board = match current_json {
            Some(json) => Self::from_json(json)?,
            None => {
                let mut scene = ExcalidrawScene::from_discovery_plan(&DiscoveryPlan {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    diagnostics: Vec::new(),
                });
                scene.elements.clear();
                Self::from_json(&scene.to_json()?)?
            }
        };
        let retired_frames: BTreeSet<_> = board
            .document
            .elements()
            .iter()
            .filter(|element| {
                !deleted(element)
                    && string(element, "type").as_deref() == Some("frame")
                    && string(element, "name")
                        .is_some_and(|name| [POOL, LEGACY_POOL, SELECTED].contains(&name.as_str()))
                    && string(element, "id").is_some_and(|id| id.starts_with("ct-planning-frame-"))
            })
            .map(required_id)
            .collect::<Result<_>>()?;
        let retired_bounds: Vec<_> = board
            .document
            .elements()
            .iter()
            .filter(|element| string(element, "id").is_some_and(|id| retired_frames.contains(&id)))
            .filter_map(ExcalidrawControlBounds::unrotated)
            .collect();
        for element in board.document.elements_mut() {
            let id = required_id(element)?;
            let in_retired_frame =
                string(element, "frameId").is_some_and(|frame| retired_frames.contains(&frame));
            let generated_control = id.starts_with("ct-planning-instructions-")
                || id.starts_with("ct-target-")
                || id.starts_with("ct-search-instructions-")
                || id.starts_with("ct-action-")
                || id.starts_with("ct-scroll-");
            let inside_retired_bounds =
                ExcalidrawControlBounds::unrotated(element).is_some_and(|child| {
                    retired_bounds
                        .iter()
                        .any(|frame| frame.contains_center(child))
                });
            if retired_frames.contains(&id)
                || ((in_retired_frame || inside_retired_bounds) && generated_control)
            {
                edit(element, |element| set(element, "isDeleted", &true))?;
            } else if in_retired_frame {
                edit(element, |element| {
                    set(element, "frameId", &Option::<String>::None)
                })?;
            }
        }
        if !board.document.elements().iter().any(|element| {
            !deleted(element)
                && string(element, "type").as_deref() == Some("frame")
                && string(element, "name").as_deref() == Some(GENERATED)
        }) {
            let x = board
                .document
                .elements()
                .iter()
                .filter(|element| !deleted(element))
                .filter_map(ExcalidrawControlBounds::unrotated)
                .map(|element| element.x + element.width)
                .reduce(f64::max)
                .map_or(40.0, |right| right + 160.0);
            let id = board.fresh_id("ct-planning-frame");
            let mut frame = rectangle(&id, x, 40.0, 1100.0, 350.0)?;
            set(&mut frame, "type", &"frame")?;
            set(&mut frame, "name", &GENERATED)?;
            board.document.elements_mut().push(frame);
        }
        board.frame(GENERATED)?;
        Ok(board)
    }

    /// Append a board beside an existing drawing, or reopen its three frames.
    /// Initial targets apply only to a newly created board: an existing empty
    /// selection is intentional and survives restarts. A partially deleted or
    /// duplicated board is diagnosed rather than silently choosing controls.
    pub fn initialize(
        current_json: Option<&str>,
        available: &[String],
        initial_selected: &[String],
        scope_label: &str,
    ) -> Result<Self> {
        let mut board = match current_json {
            Some(json) => Self::from_json(json)?,
            None => {
                let mut scene = ExcalidrawScene::from_discovery_plan(&DiscoveryPlan {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    diagnostics: Vec::new(),
                });
                scene.elements.clear();
                Self::from_json(&scene.to_json()?)?
            }
        };
        let has_board = board.document.elements().iter().any(|element| {
            !deleted(element)
                && string(element, "type").as_deref() == Some("frame")
                && string(element, "name").is_some_and(|name| {
                    [POOL, LEGACY_POOL, SELECTED, GENERATED].contains(&name.as_str())
                })
        });
        if !has_board {
            board.create_frames(initial_selected, scope_label)?;
        }
        board.migrate_legacy_pool()?;
        board.frames()?;
        board.set_scope_label(scope_label)?;
        board.reconcile_pool(available)?;
        Ok(board)
    }

    /// Target names are deduplicated and sorted, so moving or copying a card
    /// within Selected targets cannot accidentally trigger a different plan.
    /// A card is selected when its rectangle's center is inside that frame.
    pub fn selected_targets(&self) -> Result<Vec<String>> {
        let [pool, selected, _] = self.frames()?;
        let pool = bounds(&self.document.elements()[pool])?;
        let selected = bounds(&self.document.elements()[selected])?;
        let mut targets = BTreeSet::new();
        for (name, _, rectangle) in self.cards()? {
            let card = bounds(&self.document.elements()[rectangle])?;
            if selected.contains_center(card) {
                if pool.contains_center(card) {
                    bail!("target {name} is inside both Target search and Selected targets");
                }
                targets.insert(name);
            }
        }
        Ok(targets.into_iter().collect())
    }

    /// Search the catalog using the visible hand-drawn `#target` query. The
    /// resized search frame determines how many results fit, and its scrollbar
    /// selects a window into all matching targets. CT owns generated
    /// result slots, while cards moved into the selection or out of the search
    /// frame keep their identity, geometry and opaque fields. Green results
    /// acknowledge the saved selection; dragging one replenishes its slot.
    pub fn reconcile_pool(&mut self, available: &[String]) -> Result<bool> {
        let [pool, _, _] = self.frames()?;
        let pool_bounds = bounds(&self.document.elements()[pool])?;
        let selected: BTreeSet<_> = self.selected_targets()?.into_iter().collect();
        let query = self.target_query()?;
        let mut seen = BTreeSet::new();
        let mut targets = Vec::new();
        for name in available {
            validate_name(name)?;
            if seen.insert(name.clone()) {
                targets.push(name.clone());
            }
        }
        for name in &selected {
            if !seen.contains(name) {
                targets.push(name.clone());
            }
        }
        let ranked = target_search::rank(&query, &targets);
        let mut scrollbar =
            ExcalidrawScrollbar::prepare(self.document.elements_mut(), pool, HEADER_HEIGHT)?;
        let content = scrollbar.content_bounds();
        let scrollbar_elements = scrollbar.element_indices();
        let group = format!(
            "{}-target-results",
            required_id(&self.document.elements()[pool])?
        );
        let pairs = self.search_result_pairs(pool_bounds, &group)?;
        let result_elements: BTreeSet<_> = pairs
            .iter()
            .flat_map(|(label, rectangle)| [*label, *rectangle])
            .collect();
        let protected: Vec<_> = self
            .document
            .elements()
            .iter()
            .enumerate()
            .filter(|(index, element)| {
                !result_elements.contains(index)
                    && !scrollbar_elements.contains(index)
                    && !deleted(element)
                    && string(element, "type").as_deref() != Some("frame")
            })
            .map(|(_, element)| element)
            .filter_map(ExcalidrawControlBounds::read)
            .filter(|card| pool_bounds.contains_center(*card))
            .collect();
        let columns = ((content.width + GAP) / (CARD_WIDTH + GAP))
            .floor()
            .max(0.0) as usize;
        let rows = ((content.height + GAP) / (CARD_HEIGHT + GAP))
            .floor()
            .max(0.0) as usize;
        let mut positions = Vec::new();
        // Stop once every matching type has a slot; a huge resized frame must
        // not force an enormous allocation for an otherwise small catalog.
        for row in 0..if columns == 0 || ranked.is_empty() {
            0
        } else {
            rows
        } {
            for column in 0..columns {
                let position = ExcalidrawControlBounds {
                    x: content.x + column as f64 * (CARD_WIDTH + GAP),
                    y: content.y + row as f64 * (CARD_HEIGHT + GAP),
                    width: CARD_WIDTH,
                    height: CARD_HEIGHT,
                };
                if protected.iter().all(|other| !overlaps(position, *other)) {
                    positions.push(position);
                }
                if positions.len() >= ranked.len() {
                    break;
                }
            }
            if positions.len() >= ranked.len() {
                break;
            }
        }
        let offset = scrollbar.page(
            self.document.elements_mut(),
            &query,
            ranked.len(),
            positions.len(),
        )?;
        let mut changed = scrollbar.changed();
        let shown = positions.len().min(ranked.len());
        for (slot, (name, position)) in ranked.iter().skip(offset).zip(positions).enumerate() {
            let (label, rectangle) = if let Some(&pair) = pairs.get(slot) {
                pair
            } else {
                changed = true;
                self.add_card(name, pool, position.x, position.y)?
            };
            let (stroke, fill) = if selected.contains(*name) {
                ("#2b8a3e", "#d3f9d8")
            } else {
                ("#495057", "#f1f3f5")
            };
            let card_group = format!(
                "{group}-{}",
                required_id(&self.document.elements()[rectangle])?
            );
            let previous = bounds(&self.document.elements()[rectangle])?;
            changed |= edit(&mut self.document.elements_mut()[rectangle], |element| {
                set(element, "isDeleted", &false)?;
                set_number(element, "x", position.x)?;
                set_number(element, "y", position.y)?;
                set_number(element, "width", position.width)?;
                set_number(element, "height", previous.height.max(position.height))?;
                set(element, "groupIds", &vec![card_group.clone()])?;
                set(element, "strokeColor", &stroke)?;
                set(element, "backgroundColor", &fill)?;
                set(element, "fillStyle", &"solid")
            })?;
            changed |= edit(&mut self.document.elements_mut()[label], |element| {
                let content = format!("#Target\n{name}");
                if original_text(element).as_deref() != Some(&content) {
                    set(element, "text", &content)?;
                    set(element, "originalText", &content)?;
                }
                set(element, "isDeleted", &false)?;
                // Native wrapping may adjust height. Keep the result label's
                // measured text intact when its semantic content is unchanged.
                if previous.x != position.x {
                    set_number(
                        element,
                        "x",
                        number(element, "x").unwrap_or(previous.x + 12.0) + position.x - previous.x,
                    )?;
                }
                if previous.y != position.y {
                    set_number(
                        element,
                        "y",
                        number(element, "y").unwrap_or(previous.y + 12.0) + position.y - previous.y,
                    )?;
                }
                set(element, "groupIds", &vec![card_group.clone()])?;
                set(element, "strokeColor", &stroke)
            })?;
        }
        for &(label, rectangle) in pairs.iter().skip(shown) {
            for index in [label, rectangle] {
                changed |= edit(&mut self.document.elements_mut()[index], |element| {
                    set(element, "isDeleted", &true)
                })?;
            }
        }
        Ok(changed)
    }

    /// Read one visible search query in hand-drawn font 5 (Excalifont) or 1
    /// (Virgil). The lower-case marker distinguishes input from result cards.
    pub fn target_query(&self) -> Result<String> {
        let pool = bounds(&self.document.elements()[self.frames()?[0]])?;
        let controls: Vec<_> = self
            .document
            .elements()
            .iter()
            .filter(|element| !deleted(element))
            .filter(|element| {
                ExcalidrawControlBounds::unrotated(element)
                    .is_some_and(|bounds| pool.contains_center(bounds))
            })
            .filter_map(|element| {
                original_text(element)
                    .filter(|text| text.lines().next().map(str::trim) == Some("#target"))
                    .map(|text| (element, text))
            })
            .collect();
        let [(element, text)] = controls.as_slice() else {
            bail!("Target search needs exactly one hand-drawn #target query label");
        };
        bounds(element)?;
        if !matches!(number(element, "fontFamily"), Some(1.0 | 5.0)) {
            bail!("#target query must use a hand-drawn font (Excalifont or Virgil)");
        }
        Ok(text
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_owned())
    }

    /// Replace the selected set with an explicit picker result. Retained cards
    /// keep their positions. Only deselected target card pairs are deleted;
    /// annotations, action controls and other frame contents are untouched.
    pub fn replace_selected_targets(&mut self, targets: &[String]) -> Result<bool> {
        let selected = self.frames()?[1];
        let selected_bounds = bounds(&self.document.elements()[selected])?;
        let desired: BTreeSet<_> = targets.iter().cloned().collect();
        for name in &desired {
            validate_name(name)?;
        }
        self.selected_targets()?;
        let mut retained = BTreeSet::new();
        let mut changed = false;
        for (name, label, rectangle) in self.cards()? {
            if !selected_bounds.contains_center(bounds(&self.document.elements()[rectangle])?) {
                continue;
            }
            if desired.contains(&name) {
                retained.insert(name);
            } else {
                for index in [label, rectangle] {
                    changed |= edit(&mut self.document.elements_mut()[index], |element| {
                        set(element, "isDeleted", &true)
                    })?;
                }
            }
        }
        let mut occupied: Vec<_> = self
            .document
            .elements()
            .iter()
            .filter(|element| {
                !deleted(element) && string(element, "type").as_deref() != Some("frame")
            })
            .filter_map(ExcalidrawControlBounds::read)
            .filter(|element| selected_bounds.contains_center(*element))
            .collect();
        let mut next_slot = 0;
        for name in desired.difference(&retained) {
            let (x, y) = free_pool_position(selected_bounds, &occupied, &mut next_slot);
            let (_, rectangle) = self.add_card(name, selected, x, y)?;
            occupied.push(bounds(&self.document.elements()[rectangle])?);
            changed = true;
        }
        Ok(changed)
    }

    /// Replace the generated graph only when its semantic plan changes. A visible
    /// fingerprint survives process restarts and editor normalization. Reused
    /// output IDs retain opaque fields and increasing versions; obsolete slots
    /// become tombstones and are reused by later graphs instead of accumulating
    /// a new generation of elements on every edit.
    pub fn set_plan(&mut self, plan: &DiscoveryPlan) -> Result<bool> {
        let generated = self.frame(GENERATED)?;
        let frame = bounds(&self.document.elements()[generated])?;
        let frame_id = required_id(&self.document.elements()[generated])?;
        let prefix = format!("{frame_id}-plan-");
        let content = format!(
            "#CTPlan\nPlan {:016x}",
            fingerprint(&facet_json::to_string(plan)?)
        );
        let markers: Vec<_> = self
            .document
            .elements()
            .iter()
            .enumerate()
            .filter(|(_, element)| !deleted(element))
            .filter(|(_, element)| bounds(element).is_ok_and(|child| frame.contains_center(child)))
            .filter(|(_, element)| {
                original_text(element).is_some_and(|text| text.lines().next() == Some("#CTPlan"))
            })
            .map(|(index, _)| index)
            .collect();
        if markers.len() > 1 {
            bail!("Generated plan has multiple #CTPlan status labels");
        }
        let mut ungrouped = false;
        for element in self.document.elements_mut() {
            if owns_output(element, &prefix)
                && element
                    .get("groupIds")
                    .is_some_and(|groups| groups.as_str() != "[]")
            {
                ungrouped |= edit(element, |element| {
                    set(element, "groupIds", &Vec::<String>::new())
                })?;
            }
        }
        if markers.first().is_some_and(|&index| {
            original_text(&self.document.elements()[index]).as_deref() == Some(&content)
        }) {
            return Ok(ungrouped);
        }
        let mut output: Vec<SceneFields> = crate::discovery_plan_diagram::elements(plan)
            .iter()
            .map(|element| Ok(facet_json::from_str(&facet_json::to_string(element)?)?))
            .collect::<Result<_>>()?;
        let right = output
            .iter()
            .filter_map(ExcalidrawControlBounds::unrotated)
            .map(|element| element.x + element.width)
            .fold(1100.0, f64::max);
        let bottom = output
            .iter()
            .filter_map(ExcalidrawControlBounds::unrotated)
            .map(|element| element.y + element.height)
            .fold(260.0, f64::max);
        for element in &mut output {
            remap_output(element, &prefix, &frame_id, frame.x + GAP, frame.y + 70.0)?;
        }
        let mut status = crate::interactive_demo::text(
            &format!("{prefix}status"),
            &content,
            frame.x + GAP,
            frame.y + GAP,
            600.0,
            45.0,
            16.0,
        )?;
        set(&mut status, "frameId", &frame_id)?;
        set(&mut status, "groupIds", &Vec::<String>::new())?;
        output.push(status);
        let expected: BTreeSet<_> = output.iter().map(required_id).collect::<Result<_>>()?;
        for element in self.document.elements_mut().iter_mut() {
            if owns_output(element, &prefix) && !expected.contains(&required_id(element)?) {
                edit(element, |element| set(element, "isDeleted", &true))?;
            }
        }
        for generated in output {
            let id = required_id(&generated)?;
            if let Some(existing) = self
                .document
                .elements_mut()
                .iter_mut()
                .find(|element| string(element, "id").as_deref() == Some(&id))
            {
                edit(existing, |element| {
                    for (key, value) in generated {
                        if !["version", "versionNonce", "updated", "index"].contains(&key.as_str())
                        {
                            element.insert(key, value);
                        }
                    }
                    Ok(())
                })?;
            } else {
                self.document.elements_mut().push(generated);
            }
        }
        edit(&mut self.document.elements_mut()[generated], |element| {
            set(element, "width", &(right + GAP * 2.0))?;
            set(element, "height", &(bottom + 90.0))
        })?;
        Ok(true)
    }

    fn frames(&self) -> Result<[usize; 3]> {
        let mut result = [0; 3];
        for (slot, name) in [POOL, SELECTED, GENERATED].into_iter().enumerate() {
            result[slot] = self.frame(name)?;
        }
        Ok(result)
    }

    fn frame(&self, name: &str) -> Result<usize> {
        let frames: Vec<_> = self
            .document
            .elements()
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                !deleted(element)
                    && string(element, "type").as_deref() == Some("frame")
                    && string(element, "name").as_deref() == Some(name)
            })
            .map(|(index, _)| index)
            .collect();
        let [index] = frames.as_slice() else {
            bail!("planning needs exactly one native frame named {name}");
        };
        let geometry = bounds(&self.document.elements()[*index])?;
        if geometry.width <= 0.0 || geometry.height <= 0.0 {
            bail!("planning frame {name} must have positive dimensions");
        }
        required_id(&self.document.elements()[*index])?;
        Ok(*index)
    }

    fn set_scope_label(&mut self, scope_label: &str) -> Result<()> {
        let selected = self.frames()?[1];
        let selected = bounds(&self.document.elements()[selected])?;
        let labels: Vec<_> = self
            .document
            .elements()
            .iter()
            .enumerate()
            .filter(|(_, element)| !deleted(element))
            .filter(|(_, element)| {
                bounds(element).is_ok_and(|child| selected.contains_center(child))
            })
            .filter(|(_, element)| {
                original_text(element).is_some_and(|text| text.lines().next() == Some("#CTInputs"))
            })
            .map(|(index, _)| index)
            .collect();
        let [index] = labels.as_slice() else {
            bail!(
                "Selected targets needs exactly one #CTInputs label for the supplied planning inputs"
            );
        };
        let content = format!(
            "#CTInputs\n{scope_label}\nSave to update the offline plan. Move cards out to deselect."
        );
        // Excalidraw owns text wrapping; never replace its measured text merely
        // because it differs from originalText after an otherwise unchanged save.
        if original_text(&self.document.elements()[*index]).as_deref() != Some(&content) {
            edit(&mut self.document.elements_mut()[*index], |element| {
                set(element, "text", &content)?;
                set(element, "originalText", &content)
            })?;
        }
        Ok(())
    }

    fn create_frames(&mut self, initial_selected: &[String], scope_label: &str) -> Result<()> {
        let x = self
            .document
            .elements()
            .iter()
            .filter(|element| !deleted(element))
            .filter_map(ExcalidrawControlBounds::unrotated)
            .map(|element| element.x + element.width)
            .reduce(f64::max)
            .map_or(40.0, |right| right + 160.0);
        for (offset, name) in [POOL, SELECTED, GENERATED].into_iter().enumerate() {
            let id = self.fresh_id("ct-planning-frame");
            let mut element =
                rectangle(&id, x + offset as f64 * 950.0, 40.0, 850.0, SEARCH_HEIGHT)?;
            set(&mut element, "type", &"frame")?;
            set(&mut element, "name", &name)?;
            self.document.elements_mut().push(element);
        }
        let [pool, selected, _] = self.frames()?;
        for (frame, content) in [
            (pool, "#target\n".to_owned()),
            (
                selected,
                format!(
                    "#CTInputs\n{scope_label}\nSave to update the offline plan. Move cards out to deselect."
                ),
            ),
        ] {
            let geometry = bounds(&self.document.elements()[frame])?;
            let id = self.fresh_id("ct-planning-instructions");
            let mut label = crate::interactive_demo::text(
                &id,
                &content,
                geometry.x + GAP,
                geometry.y + GAP,
                810.0,
                110.0,
                16.0,
            )?;
            set(
                &mut label,
                "frameId",
                &required_id(&self.document.elements()[frame])?,
            )?;
            if frame == pool {
                set(&mut label, "fontFamily", &5)?;
                set(&mut label, "fontSize", &24.0)?;
                set(&mut label, "width", &350.0)?;
                set(&mut label, "height", &90.0)?;
            }
            self.document.elements_mut().push(label);
        }
        self.add_search_instructions(pool)?;
        for (slot, name) in initial_selected
            .iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .enumerate()
        {
            validate_name(name)?;
            let geometry = bounds(&self.document.elements()[selected])?;
            self.add_card(
                name,
                selected,
                geometry.x + GAP + (slot % 2) as f64 * (CARD_WIDTH + GAP),
                geometry.y + HEADER_HEIGHT + (slot / 2) as f64 * (CARD_HEIGHT + GAP),
            )?;
        }
        Ok(())
    }

    fn add_search_instructions(&mut self, pool: usize) -> Result<()> {
        let geometry = bounds(&self.document.elements()[pool])?;
        let id = self.fresh_id("ct-search-instructions");
        let mut label = crate::interactive_demo::text(
            &id,
            "Edit the hand-drawn #target query.\nDrag a result into Selected targets.\nDrag the purple thumb to scroll.\nGreen means selected; save to update.",
            geometry.x + 430.0,
            geometry.y + GAP,
            400.0,
            100.0,
            15.0,
        )?;
        set(
            &mut label,
            "frameId",
            &required_id(&self.document.elements()[pool])?,
        )?;
        self.document.elements_mut().push(label);
        Ok(())
    }

    fn migrate_legacy_pool(&mut self) -> Result<()> {
        let legacy: Vec<_> = self
            .document
            .elements()
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                !deleted(element)
                    && string(element, "type").as_deref() == Some("frame")
                    && string(element, "name").as_deref() == Some(LEGACY_POOL)
            })
            .map(|(index, _)| index)
            .collect();
        if legacy.is_empty() {
            return Ok(());
        }
        let [pool] = legacy.as_slice() else {
            bail!("multiple legacy Target pool frames; migration is ambiguous");
        };
        let pool = *pool;
        let geometry = bounds(&self.document.elements()[pool])?;
        // Rename first so the same visible-control validation can inspect the
        // old inventory. This edit is only published if migration succeeds.
        edit(&mut self.document.elements_mut()[pool], |element| {
            set(element, "name", &POOL)
        })?;
        for (_, label, rectangle) in self.cards()? {
            let card = bounds(&self.document.elements()[rectangle])?;
            let id = required_id(&self.document.elements()[rectangle])?;
            let column = (card.x - geometry.x - GAP) / (CARD_WIDTH + GAP);
            let row = (card.y - geometry.y - HEADER_HEIGHT) / (CARD_HEIGHT + GAP);
            let original_slot = column >= 0.0
                && row >= 0.0
                && (column - column.round()).abs() < 0.001
                && (row - row.round()).abs() < 0.001;
            if geometry.contains_center(card)
                && id.starts_with("ct-target-")
                && required_id(&self.document.elements()[label])? == format!("{id}-label")
                && original_slot
                && (card.width - CARD_WIDTH).abs() < 0.001
            {
                for index in [label, rectangle] {
                    edit(&mut self.document.elements_mut()[index], |element| {
                        set(element, "isDeleted", &true)
                    })?;
                }
            }
        }
        // Only the old, exact generated instruction is retired. User annotations
        // and moved cards remain where they were, including outside the new frame.
        for element in self.document.elements_mut().iter_mut() {
            if !deleted(element)
                && bounds(element).is_ok_and(|child| geometry.contains_center(child))
                && original_text(element).as_deref()
                    == Some(
                        "Drag a #Target card into Selected targets.\nThe pool keeps a reusable copy; green means selected.",
                    )
            {
                edit(element, |element| set(element, "isDeleted", &true))?;
            }
        }
        edit(&mut self.document.elements_mut()[pool], |element| {
            set_number(element, "width", 850.0)?;
            set_number(element, "height", SEARCH_HEIGHT)
        })?;
        let frame_id = required_id(&self.document.elements()[pool])?;
        let resized = bounds(&self.document.elements()[pool])?;
        let detached: BTreeSet<_> = self
            .document
            .elements()
            .iter()
            .filter(|element| {
                !deleted(element) && string(element, "frameId").as_ref() == Some(&frame_id)
            })
            .filter(|element| !bounds(element).is_ok_and(|child| resized.contains(child)))
            .filter_map(|element| string(element, "id"))
            .collect();
        for element in self.document.elements_mut().iter_mut() {
            if !deleted(element)
                && string(element, "frameId").as_ref() == Some(&frame_id)
                && (string(element, "id").is_some_and(|id| detached.contains(&id))
                    || string(element, "containerId").is_some_and(|id| detached.contains(&id)))
            {
                // Native frames clip their children. Detach retained notes and
                // moved cards beyond the new search bounds, keeping their
                // geometry and contents visible in the surrounding drawing.
                edit(element, |element| {
                    set(element, "frameId", &Option::<String>::None)
                })?;
            }
        }
        let id = self.fresh_id("ct-target-query");
        let mut query = crate::interactive_demo::text(
            &id,
            "#target\n",
            geometry.x + GAP,
            geometry.y + GAP,
            350.0,
            90.0,
            24.0,
        )?;
        set(&mut query, "fontFamily", &5)?;
        set(
            &mut query,
            "frameId",
            &required_id(&self.document.elements()[pool])?,
        )?;
        self.document.elements_mut().push(query);
        self.add_search_instructions(pool)?;
        Ok(())
    }

    fn search_result_pairs(
        &self,
        pool: ExcalidrawControlBounds,
        group: &str,
    ) -> Result<Vec<(usize, usize)>> {
        let mut labels = BTreeMap::<String, Vec<usize>>::new();
        for (index, element) in self.document.elements().iter().enumerate() {
            if string(element, "type").as_deref() == Some("text")
                && owns_search_result(element, group)
                && let Some(id) = string(element, "containerId")
            {
                labels.entry(id).or_default().push(index);
            }
        }
        let mut pairs = Vec::new();
        for (index, element) in
            self.document
                .elements()
                .iter()
                .enumerate()
                .filter(|(_, element)| {
                    string(element, "type").as_deref() == Some("rectangle")
                        && owns_search_result(element, group)
                })
        {
            if !bounds(element).is_ok_and(|card| pool.contains_center(card)) {
                continue;
            }
            let id = required_id(element)?;
            let Some(labels) = labels.get(&id) else {
                bail!("generated search result {id} is missing its label");
            };
            let [label] = labels.as_slice() else {
                bail!("generated search result {id} has ambiguous labels");
            };
            pairs.push((*label, index));
        }
        Ok(pairs)
    }

    // Return (target name, visible label index, enclosing rectangle index).
    fn cards(&self) -> Result<Vec<(String, usize, usize)>> {
        let [pool, selected, _] = self.frames()?;
        let pool = bounds(&self.document.elements()[pool])?;
        let selected = bounds(&self.document.elements()[selected])?;
        let mut rectangles_by_id = BTreeMap::<String, Vec<usize>>::new();
        let mut rectangles = Vec::new();
        for (index, element) in
            self.document
                .elements()
                .iter()
                .enumerate()
                .filter(|(_, element)| {
                    !deleted(element) && string(element, "type").as_deref() == Some("rectangle")
                })
        {
            rectangles_by_id
                .entry(required_id(element)?)
                .or_default()
                .push(index);
            rectangles.push(index);
        }
        let mut result = Vec::new();
        for (index, element) in self
            .document
            .elements()
            .iter()
            .enumerate()
            .filter(|(_, element)| !deleted(element))
        {
            let Some(label) = original_text(element) else {
                continue;
            };
            if label.lines().next().map(str::trim) != Some("#Target") {
                continue;
            }
            let container = string(element, "containerId");
            let container_bounds = container
                .as_ref()
                .and_then(|id| rectangles_by_id.get(id))
                .and_then(|indices| indices.first())
                .map(|&index| &self.document.elements()[index])
                .and_then(ExcalidrawControlBounds::unrotated);
            let nearby = container_bounds
                .or_else(|| ExcalidrawControlBounds::unrotated(element))
                .is_some_and(|child| {
                    pool.contains_center(child) || selected.contains_center(child)
                });
            if !nearby {
                continue;
            }
            let lines: Vec<_> = label.lines().collect();
            let [_, name] = lines.as_slice() else {
                bail!("target cards need #Target followed by one type name on the next line");
            };
            validate_name(name)?;
            let label_bounds = bounds(element)?;
            let candidates = match &container {
                Some(id) => rectangles_by_id.get(id).cloned().unwrap_or_default(),
                None => rectangles
                    .iter()
                    .copied()
                    .filter(|&index| {
                        bounds(&self.document.elements()[index])
                            .is_ok_and(|geometry| geometry.contains(label_bounds))
                    })
                    .collect(),
            };
            let [rectangle] = candidates.as_slice() else {
                bail!("target {name} needs exactly one enclosing rectangle");
            };
            let geometry = bounds(&self.document.elements()[*rectangle])?;
            if pool.contains_center(geometry) && selected.contains_center(geometry) {
                bail!("target {name} is inside both Target search and Selected targets");
            }
            if pool.contains_center(geometry) || selected.contains_center(geometry) {
                result.push((name.trim().to_owned(), index, *rectangle));
            }
        }
        // Multiple semantic labels in one rectangle are ambiguous, even if
        // currently equal; moving that rectangle must represent a single card.
        let mut rectangles = BTreeSet::new();
        for (_, _, rectangle) in &result {
            if !rectangles.insert(*rectangle) {
                bail!("a target card has multiple #Target labels");
            }
        }
        Ok(result)
    }

    fn add_card(&mut self, name: &str, frame: usize, x: f64, y: f64) -> Result<(usize, usize)> {
        let id = self.fresh_id("ct-target");
        let label_id = format!("{id}-label");
        let frame_id = required_id(&self.document.elements()[frame])?;
        let mut rectangle = rectangle(&id, x, y, CARD_WIDTH, CARD_HEIGHT)?;
        set(&mut rectangle, "frameId", &frame_id)?;
        set(
            &mut rectangle,
            "boundElements",
            &vec![BTreeMap::from([
                ("id".to_owned(), label_id.clone()),
                ("type".to_owned(), "text".to_owned()),
            ])],
        )?;
        let mut label = crate::interactive_demo::text(
            &label_id,
            &format!("#Target\n{name}"),
            x + 12.0,
            y + 12.0,
            CARD_WIDTH - 24.0,
            CARD_HEIGHT - 24.0,
            16.0,
        )?;
        set(&mut label, "frameId", &frame_id)?;
        set(&mut label, "containerId", &id)?;
        let rectangle_index = self.document.elements().len();
        self.document.elements_mut().extend([rectangle, label]);
        let geometry = bounds(&self.document.elements()[frame])?;
        edit(&mut self.document.elements_mut()[frame], |element| {
            set(
                element,
                "height",
                &geometry.height.max(y + CARD_HEIGHT + GAP - geometry.y),
            )?;
            set(
                element,
                "width",
                &geometry.width.max(x + CARD_WIDTH + GAP - geometry.x),
            )
        })?;
        Ok((rectangle_index + 1, rectangle_index))
    }

    fn fresh_id(&self, prefix: &str) -> String {
        let existing: BTreeSet<_> = self
            .document
            .elements()
            .iter()
            .filter_map(|element| string(element, "id"))
            .collect();
        (0..)
            .map(|index| format!("{prefix}-{index}"))
            .find(|id| {
                !existing.contains(id)
                    && !existing.contains(&format!("{id}-label"))
                    && !existing
                        .iter()
                        .any(|existing| existing.starts_with(&format!("{id}-plan-")))
            })
            .unwrap()
    }
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.trim() != name || name.chars().any(char::is_whitespace) {
        bail!("target type names must be nonempty and contain no whitespace: {name:?}");
    }
    Ok(())
}

fn overlaps(left: ExcalidrawControlBounds, right: ExcalidrawControlBounds) -> bool {
    left.x < right.x + right.width
        && left.x + left.width > right.x
        && left.y < right.y + right.height
        && left.y + left.height > right.y
}

fn set_number(element: &mut SceneFields, name: &str, value: f64) -> Result<()> {
    // Editors serialize integral coordinates as integers. Comparing numeric
    // values avoids treating 60 and 60.0 as a fresh edit after every save.
    if number(element, name) != Some(value) {
        set(element, name, &value)?;
    }
    Ok(())
}

fn free_pool_position(
    geometry: ExcalidrawControlBounds,
    occupied: &[ExcalidrawControlBounds],
    next_slot: &mut usize,
) -> (f64, f64) {
    let columns = (((geometry.width - GAP) / (CARD_WIDTH + GAP)).floor() as usize).max(1);
    // Continue after the previous replenishment slot instead of rescanning every
    // occupied slot for every target in a registry-sized palette.
    for _ in 0..=occupied.len().saturating_mul(columns + 2) + columns {
        let slot = *next_slot;
        *next_slot += 1;
        let x = geometry.x + GAP + (slot % columns) as f64 * (CARD_WIDTH + GAP);
        let y = geometry.y + HEADER_HEIGHT + (slot / columns) as f64 * (CARD_HEIGHT + GAP);
        if occupied.iter().all(|card| {
            x + CARD_WIDTH <= card.x
                || x >= card.x + card.width
                || y + CARD_HEIGHT <= card.y
                || y >= card.y + card.height
        }) {
            return (x, y);
        }
    }
    // Freely resized cards can cover many slots. Append below the last card.
    (
        geometry.x + GAP,
        occupied
            .iter()
            .map(|card| card.y + card.height + GAP)
            .fold(geometry.y + HEADER_HEIGHT, f64::max),
    )
}

fn bounds(element: &SceneFields) -> Result<ExcalidrawControlBounds> {
    ExcalidrawControlBounds::read(element)
        .ok_or_else(|| eyre::eyre!("planning controls must have finite, unrotated geometry"))
}

fn required_id(element: &SceneFields) -> Result<String> {
    string(element, "id")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| eyre::eyre!("planning elements require nonempty IDs"))
}

fn original_text(element: &SceneFields) -> Option<String> {
    (string(element, "type").as_deref() == Some("text"))
        .then(|| string(element, "originalText").or_else(|| string(element, "text")))
        .flatten()
}

fn rectangle(id: &str, x: f64, y: f64, width: f64, height: f64) -> Result<SceneFields> {
    let base = ExcalidrawElementBase::new(ExcalidrawElementId(id.into()), x, y, width, height);
    let mut fields = facet_json::from_str(&facet_json::to_string(&base)?)?;
    set(&mut fields, "type", &"rectangle")?;
    Ok(fields)
}

fn fingerprint(content: &str) -> u64 {
    // Fixed FNV-1a, not a process-randomized hasher. This is only a presentation
    // change detector, never authorization, signing, or a request cache key.
    content.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn owns_output(element: &SceneFields, prefix: &str) -> bool {
    string(element, "id").is_some_and(|id| id.starts_with(prefix))
        || element
            .get("groupIds")
            .and_then(|value| facet_json::from_str::<Vec<String>>(value.as_str()).ok())
            .is_some_and(|groups| groups.iter().any(|group| group == prefix))
}

fn owns_search_result(element: &SceneFields, prefix: &str) -> bool {
    element
        .get("groupIds")
        .and_then(|value| facet_json::from_str::<Vec<String>>(value.as_str()).ok())
        .is_some_and(|groups| {
            groups
                .iter()
                .any(|group| group.starts_with(&format!("{prefix}-")))
        })
}

fn remap_output(
    element: &mut SceneFields,
    prefix: &str,
    frame: &str,
    x: f64,
    y: f64,
) -> Result<()> {
    for key in ["id", "containerId"] {
        if let Some(id) = string(element, key) {
            set(element, key, &format!("{prefix}{id}"))?;
        }
    }
    for key in ["startBinding", "endBinding"] {
        if let Some(binding) = element.get(key) {
            let mut binding: SceneFields = facet_json::from_str(binding.as_str())?;
            if let Some(id) = string(&binding, "elementId") {
                set(&mut binding, "elementId", &format!("{prefix}{id}"))?;
            }
            set(element, key, &binding)?;
        }
    }
    if let Some(bindings) = element.get("boundElements") {
        let mut bindings: Vec<SceneFields> = facet_json::from_str(bindings.as_str())?;
        for binding in &mut bindings {
            if let Some(id) = string(binding, "id") {
                set(binding, "id", &format!("{prefix}{id}"))?;
            }
        }
        set(element, "boundElements", &bindings)?;
    }
    set(element, "x", &(number(element, "x").unwrap_or(0.0) + x))?;
    set(element, "y", &(number(element, "y").unwrap_or(0.0) + y))?;
    set(element, "frameId", &frame)?;
    set(element, "groupIds", &Vec::<String>::new())?;
    Ok(())
}

#[cfg(test)]
#[path = "excalidraw_planning_board_tests.rs"]
mod tests;

#[path = "planning_target_search.rs"]
mod target_search;
