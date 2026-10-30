use facet::Facet;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum FunctionKind {
    AsyncInvoke,
    Conversion,
    Projection,
    Constructor,
}
