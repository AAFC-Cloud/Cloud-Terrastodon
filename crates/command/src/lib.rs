#![feature(
    let_chains,
    duration_constructors,
    string_from_utf8_lossy_owned,
    async_fn_track_caller
)]
mod command;
mod work;
mod no_spaces;

pub use crate::command::*;
pub use crate::work::*;
pub use crate::no_spaces::*;

// TODO: add a `last_used` file to cache entries so we can give the user the list of recently used cache entries to let the user surgically bust individual caches instead of only being able to clear the entire cache
