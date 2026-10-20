use crate::EntraPrincipalKind;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::bail;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

/// Object types represented by the shallow Microsoft Graph v1.0
/// [directory-object model](https://learn.microsoft.com/en-us/graph/api/resources/directoryobject?view=graph-rest-1.0).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum EntraDirectoryObjectType {
    User,
    Group,
    ServicePrincipal,
    Device,
    OrganizationalContact,
}

impl EntraDirectoryObjectType {
    /// The resource type name used in the `types` field of `getByIds`.
    pub const fn graph_type(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Group => "group",
            Self::ServicePrincipal => "servicePrincipal",
            Self::Device => "device",
            Self::OrganizationalContact => "orgContact",
        }
    }

    /// The OData discriminator returned in `@odata.type`.
    pub const fn odata_type(self) -> &'static str {
        match self {
            Self::User => "#microsoft.graph.user",
            Self::Group => "#microsoft.graph.group",
            Self::ServicePrincipal => "#microsoft.graph.servicePrincipal",
            Self::Device => "#microsoft.graph.device",
            Self::OrganizationalContact => "#microsoft.graph.orgContact",
        }
    }

    pub fn try_from_odata_type(value: &str) -> Result<Self> {
        match value {
            "#microsoft.graph.user" => Ok(Self::User),
            "#microsoft.graph.group" => Ok(Self::Group),
            "#microsoft.graph.servicePrincipal" => Ok(Self::ServicePrincipal),
            "#microsoft.graph.device" => Ok(Self::Device),
            "#microsoft.graph.orgContact" => Ok(Self::OrganizationalContact),
            other => bail!("Unsupported Entra directory object @odata.type: {other}"),
        }
    }

    /// The corresponding full principal kind, when that model supports this type.
    ///
    /// A directory object need not be a security principal. In particular,
    /// organizational contacts cannot be represented as a full principal.
    pub const fn principal_kind(self) -> Option<EntraPrincipalKind> {
        match self {
            Self::User => Some(EntraPrincipalKind::User),
            Self::Group => Some(EntraPrincipalKind::Group),
            Self::ServicePrincipal => Some(EntraPrincipalKind::ServicePrincipal),
            Self::Device | Self::OrganizationalContact => None,
        }
    }
}

impl Display for EntraDirectoryObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.graph_type())
    }
}

cloud_terrastodon_registry::register_thing!(EntraDirectoryObjectType);
cloud_terrastodon_registry::register_arbitrary!(EntraDirectoryObjectType);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_graph_and_odata_type_names() -> eyre::Result<()> {
        assert_eq!(EntraDirectoryObjectType::User.graph_type(), "user");
        assert_eq!(
            EntraDirectoryObjectType::ServicePrincipal.odata_type(),
            "#microsoft.graph.servicePrincipal"
        );
        assert_eq!(
            EntraDirectoryObjectType::try_from_odata_type("#microsoft.graph.group")?,
            EntraDirectoryObjectType::Group
        );
        Ok(())
    }
}
