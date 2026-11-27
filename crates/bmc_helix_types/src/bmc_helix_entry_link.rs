use arbitrary::Arbitrary;
use facet::Facet;
use url::Url;

/// A link in an [AR System form response](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/BMC-Helix-Innovation-Suite/is254/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct BmcHelixEntryLink {
    #[arbitrary(with = arbitrary_link)]
    pub href: Url,
}

fn arbitrary_link(input: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Url> {
    Ok(crate::BmcHelixServerUrl::arbitrary(input)?.as_url().clone())
}
