use crate::RuntimeTestResult;
use facet::Facet;

/// A named verification's outcome. Service response records are not retained.
#[derive(Debug, Facet)]
pub struct RuntimeTestCaseReport {
    pub name: String,
    pub result: RuntimeTestResult,
}
