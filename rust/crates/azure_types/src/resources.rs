use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub kind: String,
}
impl Resource {
    pub fn name(&self) -> &str {
        self.id.rsplit_once("/").map(|x| x.1).unwrap_or(&self.id)
    }
}
