use super::OutputFormat;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use eyre::Context;
use facet::Facet;
use facet_json::to_writer_std_pretty;
use facet_pretty::ColorMode;
use facet_pretty::PrettyPrinter;
use std::io::Write;
use teamy_cancellation::CancellationScope;

pub(super) fn write_facet<T: Facet<'static>>(
    value: &T,
    format: OutputFormat,
    writer: &mut dyn Write,
    _cancellation: &CancellationScope,
) -> eyre::Result<()> {
    match format {
        OutputFormat::Text | OutputFormat::FacetPretty => {
            let plain = std::fmt::from_fn(|formatter| {
                PrettyPrinter::new()
                    .with_colors(ColorMode::Never)
                    .format_to(value, formatter)
            });
            write!(
                writer,
                "{}",
                plain.if_supports_color(Stream::Stdout, |_| {
                    std::fmt::from_fn(|formatter| {
                        PrettyPrinter::new()
                            .with_colors(ColorMode::Always)
                            .format_to(value, formatter)
                    })
                })
            )
            .wrap_err("failed to write Facet command output")
        }
        OutputFormat::Json => to_writer_std_pretty(writer, value)
            .wrap_err("failed to serialize or write command output as JSON"),
        OutputFormat::Auto => {
            unreachable!("CliOutput resolves automatic selection before invoking renderers")
        }
    }
}
