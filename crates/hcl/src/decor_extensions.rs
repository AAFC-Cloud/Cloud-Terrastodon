use hcl::edit::Decor;

pub trait DecorExtensions {
    fn is_empty(&self) -> bool;
}
impl DecorExtensions for Decor {
    fn is_empty(&self) -> bool {
        if let Some(prefix) = self.prefix() {
            if !prefix.trim().is_empty() {
                return false;
            }
        }
        if let Some(suffix) = self.suffix() {
            if !suffix.trim().is_empty() {
                return false;
            }
        }
        return true;
    }
}
