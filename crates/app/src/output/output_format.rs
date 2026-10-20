use facet::Facet;

/// The representation requested through `--output-format`.
#[derive(Facet, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[facet(rename_all = "kebab-case")]
#[repr(u8)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    /// Show the underlying data with Facet's pretty printer (`facet-pretty` or `facet`).
    #[facet(alias = "facet")]
    FacetPretty,
    /// Select text for an interactive terminal and JSON for redirected stdout.
    Auto,
}

impl OutputFormat {
    /// Resolve `auto` or an omitted format to text for terminals and JSON for pipes.
    #[must_use]
    pub const fn resolve(requested_format: Option<Self>, stdout_is_terminal: bool) -> Self {
        match requested_format {
            Some(Self::Text) => Self::Text,
            Some(Self::Json) => Self::Json,
            Some(Self::FacetPretty) => Self::FacetPretty,
            Some(Self::Auto) | None if stdout_is_terminal => Self::Text,
            Some(Self::Auto) | None => Self::Json,
        }
    }
}
