use arbitrary::Arbitrary;
use std::ops::Deref;

/// A duration accepting human-readable command-line input such as `30m`.
#[derive(facet::Facet, Debug, Clone)]
#[facet(proxy = String, opaque)]
pub struct HumantimeDurationCli(pub humantime::Duration);

impl<'a> Arbitrary<'a> for HumantimeDurationCli {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let seconds = u64::arbitrary(u)? % 86_400;
        Ok(Self(humantime::Duration::from(
            std::time::Duration::from_secs(seconds),
        )))
    }
}
impl Deref for HumantimeDurationCli {
    type Target = humantime::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for HumantimeDurationCli {
    type Error = humantime::DurationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Self)
    }
}

impl From<&HumantimeDurationCli> for String {
    fn from(value: &HumantimeDurationCli) -> Self {
        value.0.to_string()
    }
}

cloud_terrastodon_registry::register_thing!(HumantimeDurationCli);
cloud_terrastodon_registry::register_arbitrary!(HumantimeDurationCli);
