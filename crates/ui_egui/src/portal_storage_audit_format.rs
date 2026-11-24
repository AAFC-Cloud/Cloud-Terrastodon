use crate::resource_output::format_label;
use cloud_terrastodon_app::OutputFormat;

/// The storage audit's native findings view and CLI representations.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum PortalStorageAuditFormat {
    #[default]
    Findings,
    Cli(OutputFormat),
}

impl PortalStorageAuditFormat {
    pub const CHOICES: [Self; 4] = [
        Self::Findings,
        Self::Cli(OutputFormat::Json),
        Self::Cli(OutputFormat::FacetPretty),
        Self::Cli(OutputFormat::Text),
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Findings => "Findings",
            Self::Cli(format) => format_label(format),
        }
    }
}
