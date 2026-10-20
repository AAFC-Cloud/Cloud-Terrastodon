//! A consumer-owned, read-only Entra group reconciliation application.
//!
//! The input contract and reconciliation policy live here. Cloud Terrastodon
//! supplies authentication, cancellation, typed Graph requests, and output.

pub mod cli;
mod desire_document;
pub mod domain;
pub mod planning;
mod tenant_preflight;
mod validation_report;

pub use desire_document::DesireDocument;
pub use planning::DesireDocumentAchievementPlan;
pub use validation_report::ValidationReport;
