#![feature(let_chains)]
mod command;
mod errors;

pub mod prelude {
    pub use crate::command::*;
}
