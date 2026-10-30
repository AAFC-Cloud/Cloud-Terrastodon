//! Editable Excalidraw diagrams for registry discovery plans.
//!
//! This crate writes the rectangle, text, and arrow subset of the
//! [Excalidraw v2 scene format](https://docs.excalidraw.com/docs/codebase/json-schema).
//! Interactive documents preserve opaque JSON fields while editing visible controls.

mod discovery_plan_diagram;
mod discovery_plan_layout;
mod discovery_plan_layout_edge;
mod discovery_plan_layout_graph;
mod discovery_plan_layout_node;
mod discovery_plan_layout_segment;
#[cfg(test)]
mod discovery_plan_layout_tests;
mod discovery_plan_text;
mod excalidraw_action_control;
mod excalidraw_app_state;
mod excalidraw_arrow_binding;
mod excalidraw_arrowhead;
mod excalidraw_binary_file;
mod excalidraw_bound_element;
mod excalidraw_canvas_action;
mod excalidraw_control_bounds;
mod excalidraw_element;
mod excalidraw_element_base;
mod excalidraw_element_id;
mod excalidraw_file_lock;
mod excalidraw_fill_style;
mod excalidraw_interactive_document;
mod excalidraw_log_level;
mod excalidraw_log_record;
mod excalidraw_object_diagram;
mod excalidraw_object_field;
mod excalidraw_object_snapshot;
mod excalidraw_planning_board;
mod excalidraw_roundness;
mod excalidraw_row_layout;
mod excalidraw_scene;
mod excalidraw_scene_replacement;
mod excalidraw_scene_type;
mod excalidraw_scrollbar;
mod excalidraw_shared_file;
mod excalidraw_stroke_style;
mod excalidraw_text_alignment;
mod excalidraw_vertical_alignment;
mod interactive_demo;
mod interactive_logs;
mod interactive_scene_fields;
mod produce_excalidraw_dump_plan_request;

pub use excalidraw_app_state::*;
pub use excalidraw_arrow_binding::*;
pub use excalidraw_arrowhead::*;
pub use excalidraw_binary_file::*;
pub use excalidraw_bound_element::*;
pub use excalidraw_canvas_action::*;
pub use excalidraw_element::*;
pub use excalidraw_element_base::*;
pub use excalidraw_element_id::*;
pub use excalidraw_fill_style::*;
pub use excalidraw_interactive_document::*;
pub use excalidraw_log_level::*;
pub use excalidraw_log_record::*;
pub use excalidraw_object_field::*;
pub use excalidraw_object_snapshot::*;
pub use excalidraw_planning_board::*;
pub use excalidraw_roundness::*;
pub use excalidraw_scene::*;
pub use excalidraw_scene_type::*;
pub use excalidraw_shared_file::*;
pub use excalidraw_stroke_style::*;
pub use excalidraw_text_alignment::*;
pub use excalidraw_vertical_alignment::*;
pub use produce_excalidraw_dump_plan_request::*;
