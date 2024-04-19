pub enum Scope {
    ManagementGroup { name: String },
}
impl Scope {
    pub fn expanded_form(&self) -> String {
        match self {
            Scope::ManagementGroup { name } => format!(
                "/providers/Microsoft.Management/managementGroups/{}",
                name
            ),
        }
    }
    pub fn short_name(&self) -> &str {
        match self {
            Scope::ManagementGroup { name } => name,
        }
    }
}

pub trait AsScope {
    fn as_scope(&self) -> Scope;
}
