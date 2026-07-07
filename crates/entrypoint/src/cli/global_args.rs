use arbitrary::Arbitrary;
use std::path::PathBuf;

/// Arguments that apply to all commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct GlobalArgs {
    /// Enable debug logging, including backtraces on panics.
    #[facet(figue::named, default = false)]
    pub debug: bool,

    /// Log level filter directive.
    #[facet(
        figue::named,
        default = String::from("info"),
        figue::label = "DIRECTIVE",
        figue::alias = "log-level"
    )]
    pub log_filter: String,

    /// Write structured ndjson logs to this file or directory. If a directory is provided,
    /// a filename will be generated there. If omitted, no JSON log file will be written.
    #[facet(figue::named, figue::label = "FILE|DIR")]
    pub log_file: Option<PathBuf>,
}
impl<'a> Arbitrary<'a> for GlobalArgs {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            debug: bool::arbitrary(u)?,
            log_filter: String::arbitrary(u)?,
            log_file: Option::<String>::arbitrary(u)?.map(std::path::PathBuf::from),
        })
    }
}
cloud_terrastodon_registry::register_thing!(GlobalArgs);
cloud_terrastodon_registry::register_arbitrary!(GlobalArgs);
