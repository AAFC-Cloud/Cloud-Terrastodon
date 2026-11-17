//! CT's application tracing setup. Shared capture, reports, and rendering live
//! in `teamy-tracing`; this crate owns subscriber and destination policy.

mod init_tracing;

pub use init_tracing::init_tracing;
pub use init_tracing::init_tracing_with_terminal;
