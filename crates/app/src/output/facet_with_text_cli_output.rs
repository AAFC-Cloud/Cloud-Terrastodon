use super::CliOutputValue;
use super::OutputFormat;
use super::write_facet::write_facet;
use facet::Facet;
use std::io::Write;
use teamy_cancellation::CancellationScope;

pub(super) struct FacetWithTextCliOutput<T, F> {
    pub value: T,
    pub write_text: F,
}

impl<T, F> CliOutputValue for FacetWithTextCliOutput<T, F>
where
    T: Facet<'static> + 'static,
    F: Fn(&T, &mut dyn Write, &CancellationScope) -> eyre::Result<()>,
{
    fn write_to(
        &self,
        format: OutputFormat,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        if format == OutputFormat::Text {
            (self.write_text)(&self.value, writer, cancellation)
        } else {
            write_facet(&self.value, format, writer, cancellation)
        }
    }
}
