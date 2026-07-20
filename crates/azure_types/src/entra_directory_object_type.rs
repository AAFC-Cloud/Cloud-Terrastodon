use crate::PrincipalKind;
use arbitrary::Arbitrary;

/// Entra directory object types supported by the principal lookup APIs.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum EntraDirectoryObjectType {
    User,
    Group,
    ServicePrincipal,
}

impl EntraDirectoryObjectType {
    pub const PRINCIPAL_TYPES: [Self; 3] = [Self::User, Self::Group, Self::ServicePrincipal];

    /// The resource type name used in the `types` field of `getByIds`.
    pub const fn graph_type(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Group => "group",
            Self::ServicePrincipal => "servicePrincipal",
        }
    }

    /// The OData discriminator returned in `@odata.type`.
    pub const fn odata_type(self) -> &'static str {
        match self {
            Self::User => "#microsoft.graph.user",
            Self::Group => "#microsoft.graph.group",
            Self::ServicePrincipal => "#microsoft.graph.servicePrincipal",
        }
    }

    pub fn try_from_odata_type(value: &str) -> eyre::Result<Self> {
        match value {
            "#microsoft.graph.user" => Ok(Self::User),
            "#microsoft.graph.group" => Ok(Self::Group),
            "#microsoft.graph.servicePrincipal" => Ok(Self::ServicePrincipal),
            other => eyre::bail!("Unsupported Entra directory object @odata.type: {other}"),
        }
    }

    pub const fn principal_kind(self) -> PrincipalKind {
        match self {
            Self::User => PrincipalKind::User,
            Self::Group => PrincipalKind::Group,
            Self::ServicePrincipal => PrincipalKind::ServicePrincipal,
        }
    }
}

impl std::fmt::Display for EntraDirectoryObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
