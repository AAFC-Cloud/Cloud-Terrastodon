use crate::GlobalArgs;

/// The consumer owns its root schema and command dispatch.
///
/// Return the same shared global-arguments field that your Figue schema flattens.
/// App adds Figue's builtin flags around this schema when parsing, then passes
/// only your CLI to the handler. Do not declare a second builtin field.
pub trait ApplicationCli: facet::Facet<'static> {
    fn global_args(&self) -> &GlobalArgs;
}
