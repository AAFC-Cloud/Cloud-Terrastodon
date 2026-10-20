//! Command output with terminal-friendly text and pipeline-friendly JSON.

mod cli_output;
mod cli_output_value;
mod facet_cli_output;
mod facet_with_text_cli_output;
mod output_format;
mod output_writer;
mod write_facet;

pub use cli_output::CliOutput;
pub use cli_output_value::CliOutputValue;
pub use output_format::OutputFormat;

#[cfg(test)]
mod tests;
