use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::bail;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialOrd, Ord, facet::Facet)]
#[facet(json::proxy = String)]
pub struct ComputePublisherVmImageOfferSkuVersionName(CompactString);
crate::impl_facet_string_proxy!(ComputePublisherVmImageOfferSkuVersionName, value => value.to_string());
impl PartialEq for ComputePublisherVmImageOfferSkuVersionName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}
impl core::fmt::Display for ComputePublisherVmImageOfferSkuVersionName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Hash for ComputePublisherVmImageOfferSkuVersionName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_ascii_lowercase().hash(state);
    }
}
impl Slug for ComputePublisherVmImageOfferSkuVersionName {
    fn try_new(name: impl Into<CompactString>) -> eyre::Result<Self> {
        let inner = name.into();
        validate_compute_publisher_vm_image_offer_sku_version_name(&inner)?;
        Ok(Self(inner))
    }

    fn validate_slug(&self) -> eyre::Result<()> {
        validate_compute_publisher_vm_image_offer_sku_version_name(&self.0)?;
        Ok(())
    }
}
impl FromStr for ComputePublisherVmImageOfferSkuVersionName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ComputePublisherVmImageOfferSkuVersionName::try_new(s)
    }
}

fn validate_compute_publisher_vm_image_offer_sku_version_name(value: &str) -> eyre::Result<()> {
    if !(1..=256).contains(&value.len()) {
        bail!(
            "Compute Publisher VM Image Offer SKU Version Name '{}' must be between 1 and 256 characters long",
            value
        )
    }
    Ok(())
}

impl Deref for ComputePublisherVmImageOfferSkuVersionName {
    type Target = CompactString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<str> for ComputePublisherVmImageOfferSkuVersionName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> Arbitrary<'a> for ComputePublisherVmImageOfferSkuVersionName {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut s = CompactString::arbitrary(u)?;
        if s.len() > 256 {
            s.truncate(256);
        } else if s.is_empty() {
            s.push('1');
        }
        ComputePublisherVmImageOfferSkuVersionName::try_new(s)
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let name = facet_json::from_str::<ComputePublisherVmImageOfferSkuVersionName>(
            "\"latest\"",
        )?;
        assert_eq!(
            name,
            ComputePublisherVmImageOfferSkuVersionName::try_new("latest")?
        );
        assert_eq!(facet_json::to_string(&name)?, "\"latest\"");
        Ok(())
    }
}
