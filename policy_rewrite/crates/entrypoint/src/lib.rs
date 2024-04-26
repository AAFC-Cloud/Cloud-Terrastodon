#![feature(let_chains)]
mod action;
mod build_policy_imports;
mod menu;
mod process_generated;
mod tf_plan_generate_config_out;
pub mod prelude {
    pub use crate::menu::*;
}
