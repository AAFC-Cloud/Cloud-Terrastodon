#![feature(
    duration_constructors,
    string_from_utf8_lossy_owned,
    async_fn_track_caller
)]
pub mod app_work;
mod command;
mod command_kind;
mod command_output;
mod no_spaces;
mod work;

pub use crate::command::*;
pub use crate::command_kind::*;
pub use crate::command_output::*;
pub use crate::no_spaces::*;
pub use crate::work::*;

// TODO: add a `last_used` file to cache entries so we can give the user the list of recently used cache entries to let the user surgically bust individual caches instead of only being able to clear the entire cache
