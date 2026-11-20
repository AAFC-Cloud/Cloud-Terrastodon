use facet::Facet;

/// The encoded contents of a [Microsoft Graph profile photo](https://learn.microsoft.com/en-us/graph/api/profilephoto-get?view=graph-rest-1.0).
#[derive(Facet)]
pub struct EntraProfileThumbnail {
    #[facet(sensitive)]
    pub bytes: Vec<u8>,
}

cloud_terrastodon_registry::register_thing!(EntraProfileThumbnail);
