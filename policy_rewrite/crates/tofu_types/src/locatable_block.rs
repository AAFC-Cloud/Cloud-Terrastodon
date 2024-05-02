pub struct LocatableBlock {
    pub display: String,
    pub line_number: usize,
}
impl std::fmt::Display for LocatableBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display)
    }
}