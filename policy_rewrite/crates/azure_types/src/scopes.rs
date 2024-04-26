pub trait Scope {
    fn expanded_form(&self) -> &str;
    fn short_name(&self) -> &str;
}

pub trait AsScope {
    fn as_scope(&self) -> &impl Scope;
}

impl<T> AsScope for T
where
    T: Scope,
{
    fn as_scope(&self) -> &impl Scope {
        self
    }
}

impl std::fmt::Display for dyn Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded_form())
    }
}

#[derive(Debug)]
pub enum ScopeError {
    Malformed,
    InvalidName,
}
impl std::error::Error for ScopeError {}
impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ScopeError::Malformed => "Malformed",
            ScopeError::InvalidName => "Invalid Name",
        })
    }
}
