use crate::Effect;
use crate::FunctionKind;
use crate::FunctionTraitContract;
use crate::ProductionKind;
use crate::ReceiverMode;
use crate::RegistrationSite;
use crate::RuntimeFromBoxedFn;
use crate::describe_shape;
use crate::list_element_shape;
use facet::Shape;
use linkme::distributed_slice;

/// Context-free operation metadata shared by pure and executable registrations.
///
/// Inspecting this descriptor never invokes an operation or acquires runtime
/// capabilities. Effects describe the registration's declared contract rather
/// than proving that its implementation is free of additional side effects.
#[derive(Clone, Copy)]
pub struct FunctionDescriptor {
    pub input_shape: &'static Shape,
    pub output_shape: &'static Shape,
    pub receiver_mode: ReceiverMode,
    pub kind: FunctionKind,
    pub label: &'static str,
    pub trait_contract: FunctionTraitContract,
    pub effects: &'static [Effect],
    /// Invocation timing is independent of the semantic operation kind.
    pub asynchronous: bool,
    pub output_to_runtime: RuntimeFromBoxedFn,
    pub registration_site: RegistrationSite,
}

impl FunctionDescriptor {
    pub fn production_kind(&self, requested_shape: &'static Shape) -> Option<ProductionKind> {
        if self.output_shape.is_shape(requested_shape) {
            return Some(ProductionKind::Exact);
        }
        list_element_shape(self.output_shape)
            .is_some_and(|item| item.is_shape(requested_shape))
            .then_some(ProductionKind::ListElement)
    }

    pub fn supports_slot_kind(&self, slot_is_owned: bool) -> bool {
        self.receiver_mode != ReceiverMode::ByMut || slot_is_owned
    }

    pub fn is_async(&self) -> bool {
        self.asynchronous
    }

    /// A declared read without writes; unattended admission also excludes solicitation.
    pub fn is_read_only(&self) -> bool {
        self.effects.contains(&Effect::Read) && !self.effects.contains(&Effect::Write)
    }

    pub fn requires_user_input(&self) -> bool {
        self.effects.contains(&Effect::Solicit)
    }
}

pub fn describe_function(function: &FunctionDescriptor) -> String {
    format!(
        "{} {} -> {} ({})",
        function.label,
        describe_shape(function.input_shape),
        describe_shape(function.output_shape),
        function.trait_contract,
    )
}

/// References to authoritative descriptors, including executable registrations
/// linked by a higher layer. This collection never retains operation callbacks.
#[distributed_slice]
pub static KNOWN_FUNCTION_DESCRIPTORS: [&'static FunctionDescriptor];

pub fn known_function_descriptors() -> impl Iterator<Item = &'static FunctionDescriptor> {
    KNOWN_FUNCTION_DESCRIPTORS.iter().copied()
}
