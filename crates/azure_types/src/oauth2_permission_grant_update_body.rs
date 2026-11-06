/// Fields accepted by [Update oauth2PermissionGrant](https://learn.microsoft.com/graph/api/oauth2permissiongrant-update?view=graph-rest-1.0).
#[derive(facet::Facet)]
pub struct OAuth2PermissionGrantUpdateBody {
    pub scope: String,
}
