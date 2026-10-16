// Parsing schema owned by App: consumers declare only their application arguments.
#[derive(facet::Facet, Debug)]
pub(crate) struct AppCli<C> {
    #[facet(flatten)]
    pub cli: C,
    #[facet(flatten)]
    pub builtins: figue::FigueBuiltins,
}
