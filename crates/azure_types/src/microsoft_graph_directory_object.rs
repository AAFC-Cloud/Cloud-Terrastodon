#[derive(Debug, facet::Facet)]
pub struct MicrosoftGraphDirectoryObject {
    #[facet(rename = "deletedDateTime", default)]
    pub deleted_date_time: Option<chrono::DateTime<chrono::Utc>>,
}
