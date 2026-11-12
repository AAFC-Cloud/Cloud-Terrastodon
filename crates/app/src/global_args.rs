use crate::OutputFormat;
use facet::Facet;
use std::path::PathBuf;

/// Arguments that apply to all commands, including consumer-defined subcommands.
///
/// Flatten this into your own root CLI. [`crate::App::parse_from`] and
/// [`crate::App::run`] supply Figue's standard builtin flags internally.
/// Authentication is selected by service commands. Prefer
/// `Default` over exhaustive struct literals so optional facilities compose.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct GlobalArgs {
    /// Enable debug logging, including backtraces on panics.
    #[facet(figue::named, default = false)]
    pub debug: bool,

    /// Log level filter directive.
    #[facet(figue::named, default = String::from("info"), figue::label = "DIRECTIVE", figue::alias = "log-level")]
    pub log_filter: String,

    /// Log level filter directive for structured file output. If omitted, the regular log filter
    /// is also used for the file output.
    #[facet(figue::named, figue::label = "DIRECTIVE")]
    pub log_file_filter: Option<String>,

    /// Write structured ndjson logs to this file or directory. If a directory is provided,
    /// a filename will be generated there. If omitted, no JSON log file will be written.
    #[facet(figue::named, figue::label = "FILE|DIR")]
    pub log_file: Option<PathBuf>,

    /// Render command output as `text`, `json`, `facet-pretty`, or `auto`.
    /// `facet-pretty` (alias `facet`) shows the underlying data with Facet's pretty printer.
    /// With `auto` or no format, commands use text in terminals and JSON when stdout is redirected.
    #[facet(figue::named, figue::label = "FORMAT", figue::alias = "output")]
    pub output_format: Option<OutputFormat>,
}

impl Default for GlobalArgs {
    fn default() -> Self {
        Self {
            debug: false,
            log_filter: "info".into(),
            log_file_filter: None,
            log_file: None,
            output_format: None,
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for GlobalArgs {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            debug: bool::arbitrary(u)?,
            log_filter: String::arbitrary(u)?,
            log_file_filter: Option::<String>::arbitrary(u)?,
            log_file: Option::<String>::arbitrary(u)?.map(PathBuf::from),
            output_format: Option::<OutputFormat>::arbitrary(u)?,
        })
    }
}
