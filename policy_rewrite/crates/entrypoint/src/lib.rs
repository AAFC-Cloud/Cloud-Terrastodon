#![feature(let_chains)]

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
mod action;
mod build_policy_imports;
mod menu;
mod process_generated;
mod tf_plan_generate_config_out;
pub mod prelude {
    pub use crate::menu::*;
}
