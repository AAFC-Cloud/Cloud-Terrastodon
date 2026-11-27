use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;
use std::fmt;
use std::str::FromStr;

/// Visible [work-order ticket number](https://docs.helixops.ai/bin/Service-Management/IT-Service-Management/BMC-Service-Request-Management/srm2002/Integrating/Web-services-for-Work-Order-Management/GetListOfWorkOrders/).
///
/// The SDK accepts `WO` followed by ASCII digits, matching the format used
/// for queries against the standard work-order interface.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
#[facet(proxy = String)]
pub struct BmcHelixWorkOrderId(String);

impl BmcHelixWorkOrderId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for BmcHelixWorkOrderId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        Self::try_from(value.to_owned())
    }
}

impl TryFrom<String> for BmcHelixWorkOrderId {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self> {
        let Some(digits) = value.strip_prefix("WO") else {
            bail!("Helix work order ID must contain WO followed by digits");
        };
        ensure!(
            !digits.is_empty() && digits.bytes().all(|digit| digit.is_ascii_digit()),
            "Helix work order ID must contain WO followed by digits"
        );
        Ok(Self(value))
    }
}

impl From<&BmcHelixWorkOrderId> for String {
    fn from(value: &BmcHelixWorkOrderId) -> Self {
        value.0.clone()
    }
}

impl fmt::Display for BmcHelixWorkOrderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'a> Arbitrary<'a> for BmcHelixWorkOrderId {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(format!("WO{:015}", u32::arbitrary(input)?)))
    }
}

cloud_terrastodon_registry::register_thing!(BmcHelixWorkOrderId);
cloud_terrastodon_registry::register_arbitrary!(BmcHelixWorkOrderId);
