use crate::diagnostic_report_display::DiagnosticReportDisplay;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use eyre::Report;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::sync::Arc;

/// An owned error report retained until a logging or presentation surface renders it.
///
/// Pass [`Self::as_error`] as a typed tracing field so a diagnostic-aware layer
/// can retain the shared report, rather than eagerly recording its debug text.
/// Plain [`fmt::Display`] and [`fmt::Debug`] preserve the complete report while
/// removing styling. [`Self::terminal`] retains supported terminal styling.
/// Use an event field such as `error`, not `message` or a span field. Messages
/// are text; span fields render immediately to avoid retaining a report's own
/// captured span through that span's extensions.
///
/// ```
/// use cloud_terrastodon_tracing::DiagnosticReport;
///
/// let report = DiagnosticReport::new(eyre::eyre!("Could not save settings"));
/// tracing::error!(error = report.as_error(), "Settings save failed");
/// ```
#[derive(Clone)]
pub struct DiagnosticReport {
    pub report: Arc<Report>,
}

impl DiagnosticReport {
    pub fn new(report: Report) -> Self {
        Self {
            report: Arc::new(report),
        }
    }

    /// Borrow a typed tracing value that diagnostic-aware visitors can downcast.
    pub fn as_error(&self) -> &(dyn Error + 'static) {
        self
    }

    /// Render with the stderr color policy, including `NO_COLOR` and `FORCE_COLOR`.
    ///
    /// Styling is restricted to ANSI color and text-style sequences. Source-provided styling
    /// cannot be distinguished from identical sequences emitted by the report's
    /// handler; neither path may execute terminal commands or enable concealment
    /// mode.
    pub fn terminal(&self) -> impl fmt::Display + '_ {
        self.if_supports_color(Stream::Stderr, |report| DiagnosticReportDisplay { report })
    }

    pub(crate) fn format_report(
        &self,
        formatter: &mut fmt::Formatter<'_>,
        retain_styling: bool,
    ) -> fmt::Result {
        // Retain the handler and typed sources until this rendering boundary.
        // Its debug implementation includes attached sections and captured traces.
        let mut formatted = String::new();
        write!(&mut formatted, "{:?}", self.report)?;
        let mut remaining = formatted.as_str();
        let mut styled = false;
        while let Some(character) = remaining.chars().next() {
            if let Some(length) = Self::allowed_terminal_style_prefix_length(remaining) {
                if retain_styling {
                    formatter.write_str(&remaining[..length])?;
                    styled = true;
                }
                remaining = &remaining[length..];
                continue;
            }
            if character.is_control() && character != '\n' {
                write!(formatter, "{}", character.escape_default())?;
            } else {
                formatter.write_char(character)?;
            }
            remaining = &remaining[character.len_utf8()..];
        }
        if styled {
            // A source error can end with an otherwise valid, unclosed style.
            formatter.write_str("\x1b[0m")?;
        }
        Ok(())
    }

    /// Byte length of an allowed ANSI color or text-style escape at the start of `value`.
    fn allowed_terminal_style_prefix_length(value: &str) -> Option<usize> {
        let parameters = value.strip_prefix("\x1b[")?;
        let end = parameters
            .bytes()
            .position(|byte| !byte.is_ascii_digit() && byte != b';')?;
        if parameters.as_bytes()[end] != b'm' {
            return None;
        }
        let mut parameters = parameters[..end].split(';');
        while let Some(parameter) = parameters.next() {
            let parameter = if parameter.is_empty() {
                0
            } else {
                parameter.parse::<u16>().ok()?
            };
            match parameter {
                // Reset, emphasis, and foreground/background colors. Exclude
                // blinking, reverse video, concealment, and terminal mode changes.
                0..=4 | 9 | 22..=24 | 29 | 30..=37 | 39..=47 | 49 | 90..=97 | 100..=107 => {}
                38 | 48 => match parameters.next()? {
                    "5" => {
                        parameters.next()?.parse::<u8>().ok()?;
                    }
                    "2" => {
                        for _ in 0..3 {
                            parameters.next()?.parse::<u8>().ok()?;
                        }
                    }
                    _ => return None,
                },
                _ => return None,
            }
        }
        Some(end + 3)
    }
}

impl From<Report> for DiagnosticReport {
    fn from(report: Report) -> Self {
        Self::new(report)
    }
}

impl fmt::Display for DiagnosticReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_report(formatter, false)
    }
}

impl fmt::Debug for DiagnosticReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_report(formatter, false)
    }
}

// This is a tracing transport, not another cause in the error chain. Display
// already includes all sources; exposing them again would duplicate the report
// in standard tracing formatters. Typed sources remain available on `report`.
impl Error for DiagnosticReport {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    #[derive(Debug)]
    struct CountedError(Arc<AtomicUsize>);

    impl fmt::Display for CountedError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fetch_add(1, Ordering::Relaxed);
            formatter.write_str("synthetic original error")
        }
    }

    impl Error for CountedError {}

    #[test]
    fn a_shared_diagnostic_keeps_its_typed_error_without_eager_formatting() {
        let formatted = Arc::new(AtomicUsize::new(0));
        let report = Report::new(CountedError(Arc::clone(&formatted)));
        let before = formatted.load(Ordering::Relaxed);
        let diagnostic = DiagnosticReport::new(report);
        let retained = diagnostic.clone();
        let typed = diagnostic.as_error();
        let terminal = retained.terminal();

        assert_eq!(formatted.load(Ordering::Relaxed), before);
        assert!(typed.downcast_ref::<DiagnosticReport>().is_some());
        assert!(retained.report.downcast_ref::<CountedError>().is_some());
        assert!(terminal.to_string().contains("synthetic original error"));
        assert!(formatted.load(Ordering::Relaxed) > before);
    }

    #[test]
    fn plain_and_colored_reports_preserve_details_without_terminal_commands() {
        let report = DiagnosticReport::new(
            eyre::eyre!(
                "\x1b[91moriginal detail\x1b[0m\nUnicode: café 資料\n\x1b[2J\x1b]0;bad title\x07\x1b[8mhidden\r"
            )
            .wrap_err("Could not save synthetic settings"),
        );

        let plain = report.to_string();
        let colored = DiagnosticReportDisplay { report: &report }.to_string();
        for rendered in [&plain, &colored] {
            assert!(rendered.contains("Could not save synthetic settings"));
            assert!(rendered.contains("original detail"));
            assert!(rendered.contains("Unicode: café 資料\n"));
            assert!(rendered.contains("\\u{1b}[2J"));
            assert!(rendered.contains("\\u{1b}]0;bad title\\u{7}"));
            assert!(rendered.contains("\\u{1b}[8mhidden\\r"));
            assert!(!rendered.contains("\x1b[2J"));
            assert!(!rendered.contains("\x1b]0;"));
            assert!(!rendered.contains("\x1b[8m"));
        }
        assert!(!plain.contains('\x1b'));
        assert!(colored.contains("\x1b[91moriginal detail\x1b[0m"));
        assert!(colored.ends_with("\x1b[0m"));
    }

    #[test]
    fn extended_colors_do_not_admit_missing_components_or_concealment() {
        let report = DiagnosticReport::new(eyre::eyre!(
            "\x1b[38;2;1;8;255mRGB\x1b[48;5;8m256 colors\x1b[0m\n\x1b[38;2;1;2mshort\x1b[0;8mhidden"
        ));
        let colored = DiagnosticReportDisplay { report: &report }.to_string();
        assert!(colored.contains("\x1b[38;2;1;8;255mRGB"));
        assert!(colored.contains("\x1b[48;5;8m256 colors"));
        assert!(colored.contains("\\u{1b}[38;2;1;2mshort"));
        assert!(colored.contains("\\u{1b}[0;8mhidden"));
        assert!(!colored.contains("\x1b[0;8m"));
    }
}
