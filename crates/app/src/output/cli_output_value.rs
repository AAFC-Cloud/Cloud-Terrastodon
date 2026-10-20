use super::OutputFormat;
use std::io::Write;
use teamy_cancellation::CancellationScope;

/// Custom output for every format, written incrementally to the supplied writer.
///
/// Prefer [`super::CliOutput::facet_with_text`] for a text-only override with shared
/// JSON and Facet pretty output. JSON must contain data without terminal colors
/// or hyperlinks. Text renderers should use stream-aware color support and
/// detect stdout locally if they emit hyperlinks.
///
/// [`super::CliOutput`] resolves automatic format selection first. The writer checks
/// cancellation between bounded writes; producers should also check the scope
/// during computation that does not write. Already-written bytes remain written
/// when cancelled. Do not buffer an unbounded producer before writing it.
pub trait CliOutputValue {
    /// Write the selected representation without adding a final newline or flush.
    ///
    /// # Errors
    /// Returns rendering, writing, or cancellation errors.
    fn write_to(
        &self,
        format: OutputFormat,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()>;
}
