use crate::ExcalidrawAppState;
use crate::ExcalidrawBinaryFile;
use crate::ExcalidrawElement;
use crate::ExcalidrawObjectSnapshot;
use crate::ExcalidrawSceneType;
use cloud_terrastodon_registry::DiscoveryPlan;
use eyre::Context;
use facet::Facet;
use std::collections::BTreeMap;

/// An editable [Excalidraw v2 scene file](https://docs.excalidraw.com/docs/codebase/json-schema#excalidraw-files).
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
pub struct ExcalidrawScene {
    #[facet(rename = "type")]
    pub kind: ExcalidrawSceneType,
    pub version: u32,
    pub source: String,
    pub elements: Vec<ExcalidrawElement>,
    pub app_state: ExcalidrawAppState,
    pub files: BTreeMap<String, ExcalidrawBinaryFile>,
}

impl ExcalidrawScene {
    /// Draw a disposable view of observed objects. Fields, labels, and rectangles
    /// remain separately selectable; no canvas control protocol is introduced.
    pub fn from_objects(title: &str, objects: &[ExcalidrawObjectSnapshot]) -> eyre::Result<Self> {
        Ok(Self {
            kind: ExcalidrawSceneType::Excalidraw,
            version: 2,
            source: "https://github.com/AAFC-Cloud/Cloud-Terrastodon".into(),
            elements: crate::excalidraw_object_diagram::elements(title, objects)?,
            app_state: ExcalidrawAppState {
                grid_size: None,
                view_background_color: "#ffffff".into(),
            },
            files: BTreeMap::new(),
        })
    }

    /// Lay out the plan's request/type dependencies without invoking operations
    /// or inspecting seed values. IDs, positions, and serialized bytes are stable
    /// for the same ordered plan; blocked nodes are shown in red.
    pub fn from_discovery_plan(plan: &DiscoveryPlan) -> eyre::Result<Self> {
        Ok(Self {
            kind: ExcalidrawSceneType::Excalidraw,
            version: 2,
            source: "https://github.com/AAFC-Cloud/Cloud-Terrastodon".into(),
            elements: crate::discovery_plan_diagram::elements(plan)?,
            app_state: ExcalidrawAppState {
                grid_size: None,
                view_background_color: "#ffffff".into(),
            },
            files: BTreeMap::new(),
        })
    }

    pub fn to_json(&self) -> eyre::Result<String> {
        facet_json::to_string_pretty(self).wrap_err("serializing Excalidraw scene")
    }
}
