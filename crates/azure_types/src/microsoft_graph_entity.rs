use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub struct MicrosoftGraphEntity<Id> {
    pub id: Id,
}
