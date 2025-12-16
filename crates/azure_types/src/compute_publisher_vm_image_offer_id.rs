use crate::prelude::ComputePublisherName;
use crate::prelude::LocationName;
use crate::prelude::SubscriptionId;
use crate::slug::HasSlug;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::bail;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::any::type_name;
use std::str::FromStr;

use crate::compute_publisher_vm_image_offer_name::ComputePublisherVmImageOfferName;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Arbitrary)]
pub struct ComputePublisherVmImageOfferId {
    pub subscription_id: SubscriptionId,
    pub location_name: LocationName,
    pub publisher_name: ComputePublisherName,
    pub offer_name: ComputePublisherVmImageOfferName,
}
impl core::fmt::Display for ComputePublisherVmImageOfferId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "/Subscriptions/{}/Providers/Microsoft.Compute/Locations/{}/Publishers/{}/ArtifactTypes/VMImage/Offers/{}",
            self.subscription_id, self.location_name, self.publisher_name, self.offer_name
        )
    }
}
impl ComputePublisherVmImageOfferId {
    pub fn new(
        subscription_id: impl Into<SubscriptionId>,
        location_name: impl Into<LocationName>,
        publisher_name: impl Into<ComputePublisherName>,
        offer_name: impl Into<ComputePublisherVmImageOfferName>,
    ) -> ComputePublisherVmImageOfferId {
        ComputePublisherVmImageOfferId {
            subscription_id: subscription_id.into(),
            location_name: location_name.into(),
            publisher_name: publisher_name.into(),
            offer_name: offer_name.into(),
        }
    }

    pub fn try_new<S, L, P, O>(
        subscription_id: S,
        location_name: L,
        publisher_name: P,
        offer_name: O,
    ) -> eyre::Result<Self>
    where
        S: TryInto<SubscriptionId>,
        S::Error: Into<eyre::Error>,
        L: TryInto<LocationName>,
        L::Error: Into<eyre::Error>,
        P: TryInto<ComputePublisherName>,
        P::Error: Into<eyre::Error>,
        O: TryInto<ComputePublisherVmImageOfferName>,
        O::Error: Into<eyre::Error>,
    {
        let subscription_id = subscription_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert subscription_id")?;
        let location_name = location_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert location_name")?;
        let publisher_name = publisher_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert publisher_name")?;
        let offer_name = offer_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert offer_name")?;
        Ok(ComputePublisherVmImageOfferId {
            subscription_id,
            location_name,
            publisher_name,
            offer_name,
        })
    }
}

impl HasSlug for ComputePublisherVmImageOfferId {
    type Name = ComputePublisherVmImageOfferName;

    fn name(&self) -> &Self::Name {
        &self.offer_name
    }
}

impl FromStr for ComputePublisherVmImageOfferId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let us = type_name::<ComputePublisherVmImageOfferId>();
        let mut parts = s.split('/');
        if !matches!(parts.next(), Some("")) {
            bail!("Expected {us} to start with '/'",)
        }
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("subscriptions")) {
            bail!("Expected {us} to start with /Subscriptions/",)
        }
        let subscription_id = match parts.next() {
            Some(s) => s
                .parse::<SubscriptionId>()
                .wrap_err_with(|| format!("Failed to parse subscription_id part '{s}' of {us}",))?,
            None => bail!("Expected {us} to have a subscription_id part after /Subscriptions/",),
        };
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("providers")) {
            bail!("Expected {us} to have /Providers/ after subscription_id,",)
        }
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("microsoft.compute")) {
            bail!("Expected {us} to have /Providers/Microsoft.Compute/ after subscription_id,",)
        }
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("locations")) {
            bail!("Expected {us} to have /Locations/ after /Providers/Microsoft.Compute/,",)
        }
        let location_name = match parts.next() {
            Some(s) => s
                .parse::<LocationName>()
                .wrap_err_with(|| format!("Failed to parse location_name part '{s}' of {us}",))?,
            None => {
                bail!("Expected {us} to have a location_name part after /Locations/")
            }
        };
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("publishers")) {
            bail!("Expected {us} to have /Publishers/ after /Locations/{{location_name}}/")
        }
        let publisher_name = match parts.next() {
            Some(s) => s
                .parse::<ComputePublisherName>()
                .wrap_err_with(|| format!("Failed to parse publisher_name part '{s}' of {us}",))?,
            None => {
                bail!("Expected {us} to have a publisher_name part after /Publishers/,",)
            }
        };
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("artifacttypes")) {
            bail!("Expected {us} to have /ArtifactTypes/ after /Publishers/{{publisher_name}}/")
        }
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("vmimage")) {
            bail!("Expected {us} to have /VMImage/ after /ArtifactTypes/")
        }
        if !matches!(parts.next(), Some(x) if x.eq_ignore_ascii_case("offers")) {
            bail!("Expected {us} to have /Offers/ after /ArtifactTypes/VMImage/,")
        }
        let offer_name = match parts.next() {
            Some(s) => s
                .parse::<ComputePublisherVmImageOfferName>()
                .wrap_err_with(|| format!("Failed to parse offer_name part '{s}' of {us}",))?,
            None => {
                bail!("Expected {us} to have an offer_name part after /Offers/,",)
            }
        };

        Ok(ComputePublisherVmImageOfferId {
            subscription_id,
            location_name,
            publisher_name,
            offer_name,
        })
    }
}

impl Serialize for ComputePublisherVmImageOfferId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ComputePublisherVmImageOfferId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::ComputePublisherVmImageOfferId;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        // random guid
        let id = "/Subscriptions/e50a2404-2957-4a5a-83b2-5af37fe041c2/Providers/Microsoft.Compute/Locations/westus/Publishers/center-for-internet-security-inc/ArtifactTypes/VMImage/Offers/cis-almalinux";
        let parsed_id = id.parse::<ComputePublisherVmImageOfferId>()?;
        assert!(id.eq_ignore_ascii_case(&parsed_id.to_string()));
        Ok(())
    }
}
