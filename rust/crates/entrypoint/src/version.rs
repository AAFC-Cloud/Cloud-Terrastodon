pub struct Version(String);
impl Version {
    pub fn new(version: String) -> Self {
        Version(version)
    }
}
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        &self.0
    }
}