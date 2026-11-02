#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RegistrationSite {
    pub file: &'static str,
    pub line: u32,
}

impl RegistrationSite {
    pub const fn new(file: &'static str, line: u32) -> Self {
        Self { file, line }
    }
}

impl core::fmt::Display for RegistrationSite {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}
