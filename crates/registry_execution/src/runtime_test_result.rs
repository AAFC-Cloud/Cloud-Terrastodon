use crate::RuntimeTestObservation;
use facet::Facet;

/// One verification outcome, including admission and cancellation dispositions.
#[derive(Debug, Facet)]
#[repr(u8)]
pub enum RuntimeTestResult {
    Passed(RuntimeTestObservation),
    Failed { message: String },
    Skipped { reason: String },
    Cancelled { reason: String },
}
