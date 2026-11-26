use arbitrary::Arbitrary;
use facet::Facet;
use jiff::Timestamp;

#[derive(Debug, Arbitrary, Facet)]
pub struct MicrosoftGraphDirectoryObject {
    #[facet(rename = "deletedDateTime", default)]
    pub deleted_date_time: Option<Timestamp>,
}
