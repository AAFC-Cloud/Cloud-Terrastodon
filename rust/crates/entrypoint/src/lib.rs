#![feature(let_chains, iter_collect_into, duration_constructors, try_blocks)]
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
