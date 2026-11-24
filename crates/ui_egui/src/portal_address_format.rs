/// The representations of a desktop destination shown in the address bar.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum PortalAddressFormat {
    PortalUrl,
    Cli,
    Id,
}

impl PortalAddressFormat {
    pub const ALL: [Self; 3] = [Self::PortalUrl, Self::Cli, Self::Id];

    pub const fn label(self) -> &'static str {
        match self {
            Self::PortalUrl => "Portal URL",
            Self::Cli => "CLI",
            Self::Id => "ID",
        }
    }
}
