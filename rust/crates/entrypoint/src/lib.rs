#![feature(let_chains, async_closure, iter_collect_into)]
mod action;
mod actions;
mod menu;
pub mod prelude {
    pub use crate::menu::*;
}
