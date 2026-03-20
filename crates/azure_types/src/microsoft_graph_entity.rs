use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MicrosoftGraphEntity<Id> {
    pub id: Id,
}
