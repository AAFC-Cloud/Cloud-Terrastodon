use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;

/// Opaque record instance ID returned by the [Innovation Suite record API](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is251/Developing-applications-by-using-BMC-Helix-Innovation-Studio/Developing-and-deploying-code-based-applications/Customizing-an-application-with-REST-APIs/Retrieving-record-instances-using-REST-API/).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixTicketRecordId(String);

impl BmcHelixTicketRecordId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BmcHelixTicketRecordId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for BmcHelixTicketRecordId {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        ensure!(!value.is_empty(), "Helix ticket record ID is empty");
        Ok(Self(value))
    }
}

impl From<&BmcHelixTicketRecordId> for String {
    fn from(value: &BmcHelixTicketRecordId) -> Self {
        value.0.clone()
    }
}

impl<'a> Arbitrary<'a> for BmcHelixTicketRecordId {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(format!("ticket-{}", u32::arbitrary(input)?)))
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixTicketRecordId);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixTicketRecordId);
