use crate::DiagnosticReport;
use std::fmt;

/// A captured log field. Reports retain their original sources and handler until
/// a presentation surface renders them; ordinary tracing values remain text.
#[derive(Clone, Debug)]
pub enum StructuredLogValue {
    Text(String),
    Report(DiagnosticReport),
}

impl fmt::Display for StructuredLogValue {
    /// Plain presentation for file logs and GUI text surfaces.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => formatter.write_str(text),
            Self::Report(report) => fmt::Display::fmt(report, formatter),
        }
    }
}

impl From<String> for StructuredLogValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for StructuredLogValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}
