#![feature(never_type)]

mod resource_types;
mod resource_types_ext;
mod resource_types_serde;

#[cfg(test)]
mod resource_types_generator;
#[cfg(test)]
mod resource_types_tests;

pub mod prelude {
    pub use crate::resource_types::*;
}