#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) enum StorageNetworkingSection {
    #[default]
    PublicAccess,
    PrivateEndpoints,
    Routing,
    CustomDomain,
}

impl StorageNetworkingSection {
    pub const ALL: [Self; 4] = [
        Self::PublicAccess,
        Self::PrivateEndpoints,
        Self::Routing,
        Self::CustomDomain,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::PublicAccess => "Public access",
            Self::PrivateEndpoints => "Private endpoints",
            Self::Routing => "Network routing",
            Self::CustomDomain => "Custom domain",
        }
    }
}
