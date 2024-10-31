#![feature(let_chains, async_closure, iter_collect_into, duration_constructors)]
mod clap;
mod interactive;
mod menu;
mod menu_action;
mod noninteractive;
mod version;
pub mod prelude {
    pub use crate::clap::*;
    pub use crate::version::*;
}
