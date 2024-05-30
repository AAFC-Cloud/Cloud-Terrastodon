#![feature(let_chains)]
mod command;
pub mod prelude {
    pub use crate::command::*;
}
