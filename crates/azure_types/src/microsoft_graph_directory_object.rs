use arbitrary::Arbitrary;
use facet::Facet;
#[derive(Debug, Arbitrary, Facet)]
pub struct MicrosoftGraphDirectoryObject {
    #[facet(rename = "deletedDateTime", default)]
    pub deleted_date_time: Option<chrono::DateTime<chrono::Utc>>,
}
