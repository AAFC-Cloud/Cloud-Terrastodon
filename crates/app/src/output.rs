//! Command output with terminal-friendly text and pipeline-friendly JSON.

use eyre::Context;
use facet::Facet;
use facet_pretty::ColorMode;
use facet_pretty::PrettyPrinter;
use std::io;
use std::io::IsTerminal;
use std::io::Write;

/// The representation requested through `--output-format`.
#[derive(Facet, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    /// Show the underlying data with Facet's pretty printer.
    FacetPretty,
    /// Select text for an interactive terminal and JSON for redirected stdout.
    Auto,
}

impl OutputFormat {
    /// Resolve `auto` or an omitted format to text for terminals and JSON for pipes.
    #[must_use]
    pub const fn resolve(requested_format: Option<Self>, stdout_is_terminal: bool) -> Self {
        match requested_format {
            Some(Self::Text) => Self::Text,
            Some(Self::Json) => Self::Json,
            Some(Self::FacetPretty) => Self::FacetPretty,
            Some(Self::Auto) | None if stdout_is_terminal => Self::Text,
            Some(Self::Auto) | None => Self::Json,
        }
    }
}

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

/// Custom rendering for every output format.
///
/// For a text-only override with shared JSON and Facet pretty rendering, prefer
/// [`CliOutput::facet_with_text`].
///
/// JSON renderers should serialize the underlying data without terminal colors
/// or hyperlinks. Text renderers may use those facilities when
/// `stdout_is_terminal` is true.
/// [`CliOutput`] resolves automatic selection before calling this method, so
/// renderers receive [`OutputFormat::Text`], [`OutputFormat::Json`], or
/// [`OutputFormat::FacetPretty`].
pub trait CliOutputValue {
    fn render(
        &self,
        format: OutputFormat,
        stdout_is_terminal: bool,
    ) -> eyre::Result<Option<String>>;
}

struct FacetCliOutput<T> {
    value: T,
}

struct FacetWithTextCliOutput<T, F> {
    value: T,
    render_text: F,
}

impl CliOutput {
    /// Construct output with a custom renderer.
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
    #[must_use]
    pub fn facet<T>(value: T) -> Self
    where
        T: Facet<'static> + 'static,
    {
        Self::new(FacetCliOutput { value })
    }

    /// Customize text while retaining Facet's JSON and pretty representations.
    ///
    /// The callback receives the value and stdout terminal status. It runs only
    /// for text, including automatic selection in an interactive terminal.
    #[must_use]
    pub fn facet_with_text<T, F>(value: T, render_text: F) -> Self
    where
        T: Facet<'static> + 'static,
        F: Fn(&T, bool) -> eyre::Result<String> + 'static,
    {
        Self::new(FacetWithTextCliOutput { value, render_text })
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

    /// Render without writing, using the supplied stdout terminal status.
    ///
    /// An explicit representation overrides detection. An omitted format or `auto`
    /// selects text for terminals and JSON for redirected stdout, including
    /// PowerShell pipelines.
    /// This preview does not return the deferred result from [`Self::with_result`].
    ///
    /// # Errors
    ///
    /// Returns an error if the selected representation cannot be rendered.
    pub fn render(
        &self,
        requested_format: Option<OutputFormat>,
        stdout_is_terminal: bool,
    ) -> eyre::Result<Option<String>> {
        let Some(output) = &self.value else {
            return Ok(None);
        };
        let format = OutputFormat::resolve(requested_format, stdout_is_terminal);
        output.render(format, stdout_is_terminal)
    }

    /// Render and write a newline-terminated value, then return the command result.
    ///
    /// # Errors
    ///
    /// Rendering or writing errors take precedence over a deferred command error.
    /// Otherwise returns the result supplied through [`Self::with_result`], even
    /// when the command has no output.
    pub fn write_to(
        self,
        requested_format: Option<OutputFormat>,
        stdout_is_terminal: bool,
        mut writer: impl Write,
    ) -> eyre::Result<()> {
        if let Some(rendered) = self.render(requested_format, stdout_is_terminal)? {
            writer
                .write_all(rendered.as_bytes())
                .wrap_err("failed to write command output")?;
            if !rendered.ends_with('\n') {
                writer
                    .write_all(b"\n")
                    .wrap_err("failed to terminate command output")?;
            }
            writer.flush().wrap_err("failed to flush command output")?;
        }
        self.result
    }

    /// Detect stdout and emit output before returning any deferred command result.
    ///
    /// # Errors
    ///
    /// Returns rendering, stdout-writing, or deferred command errors.
    pub fn emit(self, requested_format: Option<OutputFormat>) -> eyre::Result<()> {
        let stdout = io::stdout();
        self.write_to(requested_format, stdout.is_terminal(), stdout.lock())
    }
}

impl Default for CliOutput {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> CliOutputValue for FacetCliOutput<T>
where
    T: Facet<'static> + 'static,
{
    fn render(
        &self,
        format: OutputFormat,
        stdout_is_terminal: bool,
    ) -> eyre::Result<Option<String>> {
        render_facet(&self.value, format, stdout_is_terminal).map(Some)
    }
}

impl<T, F> CliOutputValue for FacetWithTextCliOutput<T, F>
where
    T: Facet<'static> + 'static,
    F: Fn(&T, bool) -> eyre::Result<String>,
{
    fn render(
        &self,
        format: OutputFormat,
        stdout_is_terminal: bool,
    ) -> eyre::Result<Option<String>> {
        let rendered = if format == OutputFormat::Text {
            (self.render_text)(&self.value, stdout_is_terminal)?
        } else {
            render_facet(&self.value, format, stdout_is_terminal)?
        };
        Ok(Some(rendered))
    }
}

fn render_facet<T: Facet<'static>>(
    value: &T,
    format: OutputFormat,
    stdout_is_terminal: bool,
) -> eyre::Result<String> {
    match format {
        OutputFormat::Text | OutputFormat::FacetPretty => Ok(PrettyPrinter::new()
            .with_colors(if stdout_is_terminal {
                ColorMode::Always
            } else {
                ColorMode::Never
            })
            .format(value)),
        OutputFormat::Json => facet_json::to_string_pretty(value)
            .wrap_err("failed to serialize command output as JSON"),
        OutputFormat::Auto => {
            unreachable!("CliOutput resolves automatic selection before invoking renderers")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct SelectedFormat;

    impl CliOutputValue for SelectedFormat {
        fn render(
            &self,
            format: OutputFormat,
            _stdout_is_terminal: bool,
        ) -> eyre::Result<Option<String>> {
            Ok(Some(format!("{format:?}")))
        }
    }

    #[test]
    fn pipelines_default_to_json_and_explicit_formats_override_detection() {
        let output = CliOutput::new(SelectedFormat);
        for (requested, terminal, expected) in [
            (None, true, "Text"),
            (None, false, "Json"),
            (Some(OutputFormat::Json), true, "Json"),
            (Some(OutputFormat::Text), false, "Text"),
            (Some(OutputFormat::FacetPretty), true, "FacetPretty"),
            (Some(OutputFormat::FacetPretty), false, "FacetPretty"),
            (Some(OutputFormat::Auto), true, "Text"),
            (Some(OutputFormat::Auto), false, "Json"),
        ] {
            assert_eq!(
                output.render(requested, terminal).unwrap().as_deref(),
                Some(expected)
            );
        }
    }

    #[derive(Facet, Clone, Debug, PartialEq)]
    struct Report {
        name: String,
        count: u32,
    }

    #[test]
    fn custom_text_retains_generic_json_and_facet_pretty_without_calling_the_callback() {
        let report = Report {
            name: "sample".into(),
            count: 3,
        };
        let calls = Rc::new(Cell::new(0));
        let callback_calls = Rc::clone(&calls);
        let output = CliOutput::facet_with_text(report.clone(), move |report, terminal| {
            callback_calls.set(callback_calls.get() + 1);
            Ok(format!(
                "{}: {} items (terminal={terminal})",
                report.name, report.count
            ))
        });
        let generic = CliOutput::facet(report.clone());

        for format in [OutputFormat::Json, OutputFormat::FacetPretty] {
            for terminal in [true, false] {
                assert_eq!(
                    output.render(Some(format), terminal).unwrap(),
                    generic.render(Some(format), terminal).unwrap()
                );
            }
        }
        let pipeline_json = output.render(None, false).unwrap().unwrap();
        assert_eq!(
            facet_json::from_str::<Report>(&pipeline_json).unwrap(),
            report
        );
        assert_eq!(calls.get(), 0);

        for terminal in [true, false] {
            assert_eq!(
                output.render(Some(OutputFormat::Text), terminal).unwrap(),
                Some(format!("sample: 3 items (terminal={terminal})"))
            );
        }
        assert_eq!(calls.get(), 2);
        assert_eq!(
            output.render(Some(OutputFormat::Auto), true).unwrap(),
            Some("sample: 3 items (terminal=true)".into())
        );
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn custom_text_rendering_errors_propagate_without_writing() {
        let output = CliOutput::facet_with_text(
            Report {
                name: "sample".into(),
                count: 3,
            },
            |_, _| Err(eyre::eyre!("custom text rendering failed")),
        );
        let mut bytes = Vec::new();
        let error = output
            .write_to(Some(OutputFormat::Text), false, &mut bytes)
            .unwrap_err();
        assert_eq!(error.to_string(), "custom text rendering failed");
        assert!(bytes.is_empty());
    }

    #[derive(Default)]
    struct RecordingWriter {
        bytes: Vec<u8>,
        flushed: bool,
    }

    impl Write for RecordingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushed = true;
            Ok(())
        }
    }

    #[test]
    fn deferred_command_errors_follow_output_and_flush_without_changing_json_shape() {
        let report = Report {
            name: "unsuccessful response".into(),
            count: 3,
        };
        let output = CliOutput::facet(report.clone())
            .with_result(Err(eyre::eyre!("request was unsuccessful")));
        // A preview remains available before emission returns the command result.
        let preview = output
            .render(Some(OutputFormat::Json), false)
            .unwrap()
            .unwrap();
        assert_eq!(
            Some(preview.clone()),
            CliOutput::facet(report.clone())
                .render(Some(OutputFormat::Json), false)
                .unwrap()
        );
        assert_eq!(facet_json::from_str::<Report>(&preview).unwrap(), report);

        let mut writer = RecordingWriter::default();
        let error = output.write_to(None, false, &mut writer).unwrap_err();
        assert_eq!(error.to_string(), "request was unsuccessful");
        assert!(writer.flushed);
        assert!(writer.bytes.ends_with(b"\n"));
        let json = String::from_utf8(writer.bytes).unwrap();
        assert_eq!(json, format!("{preview}\n"));
        assert_eq!(facet_json::from_str::<Report>(&json).unwrap(), report);
    }

    #[test]
    fn deferred_command_errors_are_returned_when_there_is_no_output() {
        let mut bytes = Vec::new();
        let error = CliOutput::none()
            .with_result(Err(eyre::eyre!("command failed without output")))
            .write_to(None, false, &mut bytes)
            .unwrap_err();
        assert_eq!(error.to_string(), "command failed without output");
        assert!(bytes.is_empty());
    }

    #[test]
    fn json_is_valid_and_has_no_terminal_escapes_even_on_a_terminal() {
        for terminal in [true, false] {
            let report = Report {
                name: "a \"quoted\" name\nwith another line".into(),
                count: 3,
            };
            let output = CliOutput::facet(Report {
                name: report.name.clone(),
                count: report.count,
            });
            let json = output
                .render(Some(OutputFormat::Json), terminal)
                .unwrap()
                .unwrap();
            assert!(!json.contains('\u{1b}'), "{json}");
            assert_eq!(facet_json::from_str::<Report>(&json).unwrap(), report);
        }
    }

    #[test]
    fn redirected_text_has_no_terminal_escapes() {
        let text = CliOutput::facet(Report {
            name: "sample".into(),
            count: 3,
        })
        .render(Some(OutputFormat::Text), false)
        .unwrap()
        .unwrap();
        assert!(text.contains("sample"), "{text}");
        assert!(!text.contains('\u{1b}'), "{text}");
    }

    #[test]
    fn facet_pretty_shows_fields_and_only_colors_terminal_output() {
        for terminal in [true, false] {
            let text = CliOutput::facet(Report {
                name: "sample".into(),
                count: 3,
            })
            .render(Some(OutputFormat::FacetPretty), terminal)
            .unwrap()
            .unwrap();
            assert!(text.contains("name"), "{text}");
            assert!(text.contains("sample"), "{text}");
            assert!(text.contains("count"), "{text}");
            assert_eq!(text.contains('\u{1b}'), terminal, "{text}");
        }
    }

    #[test]
    fn none_has_no_output_in_any_format() {
        for format in [
            None,
            Some(OutputFormat::Text),
            Some(OutputFormat::Json),
            Some(OutputFormat::FacetPretty),
        ] {
            assert_eq!(CliOutput::none().render(format, true).unwrap(), None);
        }
    }
}
