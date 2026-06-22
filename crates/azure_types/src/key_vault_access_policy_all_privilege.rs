/// Shared "All" privilege used across key/secret/certificate access policy privilege enums
/// to reduce duplication. Facet decodes common variants of "All" case-insensitively.
#[derive(Debug, PartialEq, Clone, Copy, Default, facet::Facet)]
#[repr(C)]
pub enum KeyVaultAccessPolicyAllPrivilege {
    #[default]
    All,
}
