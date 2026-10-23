//! Editable Excalidraw diagrams for registry discovery plans.
//!
//! This crate writes the rectangle, text, and arrow subset of the
//! [Excalidraw v2 scene format](https://docs.excalidraw.com/docs/codebase/json-schema).
//! It is an exporter, not a lossless reader for arbitrary editor documents.

mod discovery_plan_diagram;
mod discovery_plan_layout;
mod discovery_plan_layout_edge;
mod discovery_plan_layout_graph;
mod discovery_plan_layout_node;
mod discovery_plan_layout_segment;
#[cfg(test)]
mod discovery_plan_layout_tests;
mod discovery_plan_text;
mod excalidraw_app_state;
mod excalidraw_arrow_binding;
mod excalidraw_arrowhead;
mod excalidraw_binary_file;
mod excalidraw_bound_element;
mod excalidraw_element;
mod excalidraw_element_base;
mod excalidraw_element_id;
mod excalidraw_fill_style;
mod excalidraw_roundness;
mod excalidraw_scene;
mod excalidraw_scene_type;
mod excalidraw_stroke_style;
mod excalidraw_text_alignment;
mod excalidraw_vertical_alignment;

pub use excalidraw_app_state::*;
pub use excalidraw_arrow_binding::*;
pub use excalidraw_arrowhead::*;
pub use excalidraw_binary_file::*;
pub use excalidraw_bound_element::*;
pub use excalidraw_element::*;
pub use excalidraw_element_base::*;
pub use excalidraw_element_id::*;
pub use excalidraw_fill_style::*;
pub use excalidraw_roundness::*;
pub use excalidraw_scene::*;
pub use excalidraw_scene_type::*;
pub use excalidraw_stroke_style::*;
pub use excalidraw_text_alignment::*;
pub use excalidraw_vertical_alignment::*;
