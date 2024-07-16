#![feature(let_chains, async_closure, iter_collect_into, duration_constructors)]
mod clap;
mod interactive;
mod menu_action;
mod menu;
mod noninteractive;
mod read_line;
mod version;
pub mod prelude {
    pub use crate::clap::*;
    pub use crate::version::*;
}
