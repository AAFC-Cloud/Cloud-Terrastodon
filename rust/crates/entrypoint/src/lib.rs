#![feature(let_chains)]
mod action;
mod actions;
mod menu;
pub mod prelude {
    pub use crate::menu::*;
}
