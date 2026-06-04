use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum GiteaRepoEnumerationMethod {
    Organizations,
    Users,
    CurrentUser,
    Search,
    IdRange,
    Combined,
}
