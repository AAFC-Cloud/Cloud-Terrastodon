use arbitrary::Arbitrary;

#[derive(Debug, Clone, Copy, Default, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[repr(u8)]
pub enum AzureDevOpsWorkItemExpand {
    None,
    Relations,
    Fields,
    Links,
    #[default]
    All,
}

impl AzureDevOpsWorkItemExpand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Relations => "relations",
            Self::Fields => "fields",
            Self::Links => "links",
            Self::All => "all",
        }
    }
}
