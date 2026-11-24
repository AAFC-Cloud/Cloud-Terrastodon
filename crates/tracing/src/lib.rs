//! CT's application tracing setup. Shared capture, reports, and rendering live
//! in `teamy-tracing`; this crate owns subscriber and destination policy.

mod figue_default_extraction_terminal_filter;
mod init_tracing;
mod simple_css_terminal_filter;
mod simple_css_warning;

pub use init_tracing::init_tracing;
pub use init_tracing::init_tracing_with_terminal;
