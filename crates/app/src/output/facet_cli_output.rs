use super::CliOutputValue;
use super::OutputFormat;
use super::write_facet::write_facet;
use facet::Facet;
use std::io::Write;
use teamy_cancellation::CancellationScope;

pub(super) struct FacetCliOutput<T> {
    pub value: T,
}

impl<T> CliOutputValue for FacetCliOutput<T>
where
    T: Facet<'static> + 'static,
{
    fn write_to(
        &self,
        format: OutputFormat,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        write_facet(&self.value, format, writer, cancellation)
    }
}
