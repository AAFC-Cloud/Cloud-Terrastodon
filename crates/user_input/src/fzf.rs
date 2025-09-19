// use crate::PickerTui;
// use eyre::Context;
// use eyre::ContextCompat;
// use eyre::Error;
// use eyre::Result;
// use eyre::eyre;
// use indexmap::IndexSet;
// use itertools::Itertools;
// use std::ffi::OsStr;
// use std::fmt::Display;
// use std::io::ErrorKind;
// use std::io::Write;
// use std::ops::Deref;
// use std::process::Command;
// use std::process::Stdio;

// #[derive(Debug)]
// pub struct FzfArgs<T> {
//     /// The list of items being chosen from
//     pub choices: Vec<T>,

//     /// The term that appears before the user's text input
//     pub prompt: Option<String>,

//     /// The term that appears between the item list and the user's text input
//     pub header: Option<String>,

//     /// The default search term
//     pub query: Option<String>,
// }
// impl<T> Default for FzfArgs<T> {
//     fn default() -> Self {
//         Self {
//             choices: Default::default(),
//             prompt: Default::default(),
//             header: Default::default(),
//             query: Default::default(),
//         }
//     }
// }
// impl<T> From<Vec<T>> for FzfArgs<T> {
//     fn from(value: Vec<T>) -> Self {
//         FzfArgs {
//             choices: value,
//             ..Default::default()
//         }
//     }
// }

// /// Prompt the user to pick from a predetermined list of options.
// pub fn pick<T>(args: impl Into<FzfArgs<T>>) -> Result<T>
// where
//     T: Into<Choice<T>>,
// {
//     let args: FzfArgs<T> = args.into();
//     let mut tui = PickerTui::new(args.choices);
//     tui.header = args.header;
//     tui.pick_one()?.ok()
// }

// /// Prompt the user to pick from a predetermined list of options.
// pub fn pick_many<T>(args: impl Into<FzfArgs<T>>) -> Result<Vec<T>>
// where
//     T: Into<Choice<T>>,
// {
//     // outer(args, ["--multi"])
//     let args = args.into();
//     let mut tui = PickerTui::new(args.choices);
//     tui.header = args.header;
//     tui.pick_many()?.ok()
// }
