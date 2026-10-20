use arbitrary::Arbitrary;
use uuid::Uuid;

/// The directory object ID (`id`) of an Entra [device](https://learn.microsoft.com/en-us/graph/api/resources/device?view=graph-rest-1.0#properties).
///
/// This identifies the directory object, rather than the distinct `deviceId`
/// alternate key assigned by the Device Registration Service.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, facet::Facet)]
#[facet(json::proxy = String)]
pub struct EntraDeviceObjectId(Uuid);

crate::impl_uuid_newtype!(EntraDeviceObjectId);

cloud_terrastodon_registry::register_thing!(EntraDeviceObjectId);
cloud_terrastodon_registry::register_arbitrary!(EntraDeviceObjectId);
