use super::CliOutputValue;
use super::OutputFormat;
use super::facet_cli_output::FacetCliOutput;
use super::facet_with_text_cli_output::FacetWithTextCliOutput;
use super::output_writer::OutputWriter;
use eyre::Context;
use facet::Facet;
use std::io;
use std::io::IsTerminal;
use std::io::Write;
use teamy_cancellation::CancellableWriter;
use teamy_cancellation::CancellationReportExt;
use teamy_cancellation::CancellationScope;

/// A command's output and optional result to return after emission.
pub struct CliOutput {
    value: Option<Box<dyn CliOutputValue>>,
    result: eyre::Result<()>,
}

impl core::fmt::Debug for CliOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CliOutput")
            .field("has_value", &self.value.is_some())
            .field("has_error", &self.result.is_err())
            .finish()
    }
}

impl CliOutput {
    /// Construct output with a custom streaming renderer.
    #[must_use]
    pub fn new(value: impl CliOutputValue + 'static) -> Self {
        Self {
            value: Some(Box::new(value)),
            result: Ok(()),
        }
    }

    /// Return no output, for commands which already perform their own interaction.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            value: None,
            result: Ok(()),
        }
    }

    /// Use Facet's pretty printer for text/`facet-pretty` and serializer for JSON.
    ///
    /// JSON and pretty output stream through the cancellation-aware writer.
    /// Checkpoints honor the supplied deferrals and occur between bounded writes;
    /// computation that does not write and blocked synchronous writes cannot be
    /// interrupted. Failures can leave a partial output prefix.
    #[must_use]
    pub fn facet<T>(value: T) -> Self
    where
        T: Facet<'static> + 'static,
    {
        Self::new(FacetCliOutput { value })
    }

    /// Customize text while retaining Facet's JSON and pretty representations.
    ///
    /// The callback receives the value, writer, and operation's cancellation
    /// scope. It runs only for text and writes incrementally. The writer checks
    /// cancellation between bounded writes; check the scope during computation
    /// that does not write. Emission owns the final newline and flush, and an
    /// error may leave an already-written prefix.
    /// Use stream-aware styling to honor color preferences independently of
    /// automatic format selection. Use [`CliOutputValue`] to customize every format.
    #[must_use]
    pub fn facet_with_text<T, F>(value: T, write_text: F) -> Self
    where
        T: Facet<'static> + 'static,
        F: Fn(&T, &mut dyn Write, &CancellationScope) -> eyre::Result<()> + 'static,
    {
        Self::new(FacetWithTextCliOutput { value, write_text })
    }

    /// Return a command result after its output has been written and flushed.
    ///
    /// This supports commands such as REST requests which show an unsuccessful
    /// response before returning an error. The result is not part of the output.
    #[must_use]
    pub fn with_result(mut self, result: eyre::Result<()>) -> Self {
        self.result = result;
        self
    }

    /// Collect a finite output preview without adding a newline or flushing.
    ///
    /// An omitted format or `auto` selects text for terminals and JSON for pipes.
    /// Terminal status controls format selection; renderers detect color support
    /// from stdout and the environment. This method buffers the whole output and
    /// ignores the result from [`Self::with_result`]. Use [`Self::write_to`] for
    /// streaming producers.
    ///
    /// # Errors
    /// Returns rendering, cancellation, or invalid UTF-8 errors.
    pub fn render(
        &self,
        requested_format: Option<OutputFormat>,
        stdout_is_terminal: bool,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Option<String>> {
        let Some(output) = &self.value else {
            return Ok(None);
        };
        let format = OutputFormat::resolve(requested_format, stdout_is_terminal);
        let mut bytes = Vec::new();
        cancellation.bail_if_cancelled()?;
        output
            .write_to(
                format,
                &mut CancellableWriter::new(&mut bytes, cancellation),
                cancellation,
            )
            .map_err(CancellationReportExt::normalize_cancellation)?;
        cancellation.bail_if_cancelled()?;
        String::from_utf8(bytes)
            .map(Some)
            .wrap_err("command output was not valid UTF-8")
    }

    /// Stream a newline-terminated value, flush, then return the command result.
    ///
    /// Cancellation is checked between bounded writes and before flushing.
    /// Already-written bytes remain written, so cancellation may leave a partial
    /// document. A blocked synchronous write or flush cannot be interrupted.
    ///
    /// # Errors
    /// Rendering, cancellation, and writer errors take precedence over a deferred
    /// command error. Otherwise returns [`Self::with_result`]'s result, including
    /// when the command has no output.
    pub fn write_to(
        self,
        requested_format: Option<OutputFormat>,
        stdout_is_terminal: bool,
        mut writer: impl Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        if let Some(output) = self.value {
            let format = OutputFormat::resolve(requested_format, stdout_is_terminal);
            let mut writer = OutputWriter::new(&mut writer, cancellation);
            cancellation.bail_if_cancelled()?;
            output
                .write_to(format, &mut writer, cancellation)
                .map_err(CancellationReportExt::normalize_cancellation)?;
            writer
                .finish()
                .map_err(CancellationReportExt::normalize_cancellation)?;
        }
        self.result
    }

    /// Detect stdout and emit output before returning any deferred command result.
    ///
    /// Acquiring stdout's synchronous lock cannot be interrupted by cancellation.
    ///
    /// # Errors
    /// Returns rendering, cancellation, stdout-writing, or deferred command errors.
    pub fn emit(
        self,
        requested_format: Option<OutputFormat>,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        let stdout = io::stdout();
        self.write_to(
            requested_format,
            stdout.is_terminal(),
            stdout.lock(),
            cancellation,
        )
    }
}

impl Default for CliOutput {
    fn default() -> Self {
        Self::none()
    }
}
