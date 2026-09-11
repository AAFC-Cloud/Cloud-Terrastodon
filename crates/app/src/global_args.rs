use std::path::PathBuf;

#[cfg(feature = "auth")]
use cloud_terrastodon_credentials::AuthSource;

/// Arguments that apply to all commands, including consumer-defined subcommands.
///
/// Flatten this into your own root CLI alongside `figue::FigueBuiltins`.
/// Authentication selection is available with the `auth` feature. Prefer
/// `Default` over exhaustive struct literals so optional facilities compose.
#[derive(facet::Facet, Debug, Clone, PartialEq, Eq)]
pub struct GlobalArgs {
    /// Authentication source preference. `auto` uses workload identity when
    /// configured and otherwise defaults to Azure CLI. Browser authentication
    /// can be selected explicitly or configured per tenant.
    #[cfg(feature = "auth")]
    #[facet(figue::named, default = AuthSource::default(), figue::label = "SOURCE")]
    pub auth_source: AuthSource,

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
}

impl Default for GlobalArgs {
    fn default() -> Self {
        Self {
            #[cfg(feature = "auth")]
            auth_source: AuthSource::default(),
            debug: false,
            log_filter: "info".into(),
            log_file_filter: None,
            log_file: None,
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for GlobalArgs {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            #[cfg(feature = "auth")]
            auth_source: AuthSource::arbitrary(u)?,
            debug: bool::arbitrary(u)?,
            log_filter: String::arbitrary(u)?,
            log_file_filter: Option::<String>::arbitrary(u)?,
            log_file: Option::<String>::arbitrary(u)?.map(PathBuf::from),
        })
    }
}
