use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet::Facet;

/// The encoded contents of a [Microsoft Graph profile photo](https://learn.microsoft.com/en-us/graph/api/profilephoto-get?view=graph-rest-1.0).
#[derive(Facet)]
pub struct EntraProfileThumbnail {
    #[facet(sensitive)]
    pub bytes: Vec<u8>,
}

impl<'a> Arbitrary<'a> for EntraProfileThumbnail {
    fn arbitrary(_input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // A synthetic 1×1 gray PNG keeps simulated photos decodable without an image encoder.
        const SYNTHETIC_PHOTO: &[u8] = b"\
            \x89PNG\r\n\x1a\n\
            \x00\x00\x00\x0dIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x00\x00\x00\x00\x3a\x7e\x9b\x55\
            \x00\x00\x00\x0dIDAT\x78\x01\x01\x02\x00\xfd\xff\x00\x80\x00\x82\x00\x81\xc3\x6e\x25\xe0\
            \x00\x00\x00\x00IEND\xae\x42\x60\x82";
        Ok(Self {
            bytes: SYNTHETIC_PHOTO.to_vec(),
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraProfileThumbnail);
cloud_terrastodon_registry::register_arbitrary!(EntraProfileThumbnail);
cloud_terrastodon_registry::register_arbitrary!(Option<EntraProfileThumbnail>);
