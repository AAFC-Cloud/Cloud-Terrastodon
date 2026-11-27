use arbitrary::Arbitrary;
use facet::Facet;
use std::fmt;
use std::num::ParseIntError;

/// Numeric field identifier in the [Shared Ticket Console record definition](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Helix-ITSM/itsm262/Administering/Enabling-and-customizing-Shared-Ticket-Console/Customizing-Shared-Ticket-Console/).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixTicketFieldId(u32);

impl BmcHelixTicketFieldId {
    pub const RECORD_ID: Self = Self(379);
    pub const TICKET_NUMBER: Self = Self(488100470);
    pub const SUMMARY: Self = Self(488100240);
    pub const STATUS: Self = Self(488100098);
    pub const SUPPORT_GROUP: Self = Self(488100050);
    pub const ASSIGNEE: Self = Self(488100080);

    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for BmcHelixTicketFieldId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl TryFrom<String> for BmcHelixTicketFieldId {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Self)
    }
}

impl From<&BmcHelixTicketFieldId> for String {
    fn from(value: &BmcHelixTicketFieldId) -> Self {
        value.to_string()
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixTicketFieldId);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixTicketFieldId);
