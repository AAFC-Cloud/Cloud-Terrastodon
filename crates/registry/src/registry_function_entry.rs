use crate::Effect;
use crate::Function;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::ReceiverMode;
use crate::describe_shape;
use facet::Facet;

/// An owned, serializable description of an operation registration.
///
/// Capturing this metadata never invokes the operation, constructs its input,
/// or resolves credentials. Effects are declarations to audit, rather than
/// proof that an implementation has no other side effects. Empty effects are
/// valid for pure constructors, conversions, and projections.
#[derive(Debug, Clone, Facet)]
pub struct RegistryFunctionEntry {
    pub input: String,
    pub output: String,
    pub kind: FunctionKind,
    pub effects: Vec<Effect>,
    pub receiver_mode: ReceiverMode,
    pub label: String,
    pub trait_contract: FunctionTraitContract,
    pub registration_file: String,
    pub registration_line: u32,
}

impl From<&Function> for RegistryFunctionEntry {
    fn from(function: &Function) -> Self {
        Self {
            input: describe_shape(function.input_shape),
            output: describe_shape(function.output_shape),
            kind: function.kind,
            effects: function.effects.to_vec(),
            receiver_mode: function.receiver_mode,
            label: function.label.to_owned(),
            trait_contract: function.trait_contract,
            registration_file: function.registration_site.file.to_owned(),
            registration_line: function.registration_site.line,
        }
    }
}
