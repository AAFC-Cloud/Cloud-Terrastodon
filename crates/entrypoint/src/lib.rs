#![feature(iter_collect_into, try_blocks)]
mod clap;
mod entrypoint;
mod interactive;
mod menu;
mod menu_action;
mod noninteractive;
mod version;

pub mod tracing;
pub(crate) mod windows_support;
pub mod prelude {
    pub use crate::clap::*;
    pub use crate::entrypoint::*;
    pub use crate::version::*;
}
