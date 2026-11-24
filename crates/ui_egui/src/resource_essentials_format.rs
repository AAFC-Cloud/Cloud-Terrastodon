use crate::resource_output::format_label;
use cloud_terrastodon_app::OutputFormat;

/// The resource representations available in the desktop Essentials section.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ResourceEssentialsFormat {
    #[default]
    Azure,
    Tree,
    Cli(OutputFormat),
}

impl ResourceEssentialsFormat {
    pub const CHOICES: [Self; 5] = [
        Self::Azure,
        Self::Tree,
        Self::Cli(OutputFormat::Json),
        Self::Cli(OutputFormat::FacetPretty),
        Self::Cli(OutputFormat::Text),
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Azure => "Azure",
            Self::Tree => "Tree",
            Self::Cli(format) => format_label(format),
        }
    }
}
