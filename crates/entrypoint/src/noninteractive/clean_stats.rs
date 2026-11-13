/// Files and bytes physically removed from generated directories.
/// Cache invalidation retains evidence and does not contribute to these totals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CleanStats {
    pub files: u64,
    pub bytes: u64,
}

impl CleanStats {
    pub(super) fn add_assign(&mut self, other: Self) {
        self.files = self.files.saturating_add(other.files);
        self.bytes = self.bytes.saturating_add(other.bytes);
    }
}
