#![forbid(unsafe_code)]
#![allow(clippy::needless_range_loop)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(all(feature = "ratatui", feature = "tuirs"))]
compile_error!(
    "ratatui support and tui-rs support are exclusive. only one of them can be enabled at the same time. see https://github.com/rhysd/tui-textarea#installation"
);

mod cursor;
mod highlight;
mod history;
mod input;
mod scroll;
#[cfg(feature = "search")]
mod search;
mod textarea;
mod util;
mod widget;
mod word;

#[cfg(feature = "crossterm")]
#[allow(clippy::single_component_path_imports)]
use crossterm;
#[cfg(feature = "tuirs-crossterm")]
use crossterm_025 as crossterm;
pub use cursor::CursorMove;
pub use input::Input;
pub use input::Key;
#[cfg(feature = "ratatui")]
#[allow(clippy::single_component_path_imports)]
use ratatui;
pub use scroll::Scrolling;
#[cfg(feature = "termion")]
#[allow(clippy::single_component_path_imports)]
use termion;
#[cfg(feature = "tuirs-termion")]
use termion_15 as termion;
pub use textarea::TextArea;
#[cfg(feature = "tuirs")]
use tui as ratatui;
