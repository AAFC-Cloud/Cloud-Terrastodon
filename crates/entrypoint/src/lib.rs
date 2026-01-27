#![feature(iter_collect_into, try_blocks)]
mod cli;
mod entrypoint;
mod git_revision;
mod interactive;
mod menu;
mod menu_action;
mod noninteractive;
mod version;

pub(crate) mod windows_support;

pub mod prelude {
    pub use crate::cli::prelude::*;
    pub use crate::entrypoint::*;
    pub use crate::version::*;
    pub use crate::git_revision::GitRevision;
}
