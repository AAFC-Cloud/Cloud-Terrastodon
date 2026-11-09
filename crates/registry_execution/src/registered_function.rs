use crate::Function;
use crate::known_functions;
use cloud_terrastodon_registry::RegisteredShape;
use facet::Facet;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;
use std::sync::LazyLock;

/// A reflected reference to one registered operation, including its executor.
///
/// Lists of these handles let a request constrain its operation catalog without
/// copying metadata or exposing function pointers as editable fields. The string
/// representation identifies the signature and registration site in this build;
/// it is not a persistent operation ID across source changes.
#[derive(Clone, Copy, Facet)]
#[facet(proxy = String)]
pub struct RegisteredFunction(#[facet(opaque)] Function);

impl RegisteredFunction {
    pub const fn from_function(function: Function) -> Self {
        Self(function)
    }

    pub const fn function(self) -> Function {
        self.0
    }
}

impl PartialEq for RegisteredFunction {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RegisteredFunction {}

impl Hash for RegisteredFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Display for RegisteredFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} -> {} ({}) @ {}",
            self.0.label,
            RegisteredShape::from_shape(self.0.input_shape),
            RegisteredShape::from_shape(self.0.output_shape),
            self.0.trait_contract,
            self.0.registration_site,
        )
    }
}

impl fmt::Debug for RegisteredFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl FromStr for RegisteredFunction {
    type Err = eyre::Report;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        // Parsing is the opt-in boundary to the immutable linked registry.
        // Planning itself uses only the handles supplied by its caller.
        static LINKED_FUNCTIONS: LazyLock<HashMap<String, Option<RegisteredFunction>>> =
            LazyLock::new(|| {
                let mut by_name = HashMap::new();
                for function in known_functions()
                    .iter()
                    .map(RegisteredFunction::from_function)
                {
                    by_name
                        .entry(function.to_string())
                        .and_modify(|existing| {
                            if *existing != Some(function) {
                                *existing = None;
                            }
                        })
                        .or_insert(Some(function));
                }
                by_name
            });
        match LINKED_FUNCTIONS.get(name) {
            Some(Some(function)) => Ok(*function),
            Some(None) => Err(eyre::eyre!(
                "Registered function name {name:?} is ambiguous"
            )),
            None => Err(eyre::eyre!("No registered function matches {name:?}")),
        }
    }
}

impl TryFrom<String> for RegisteredFunction {
    type Error = eyre::Report;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        name.parse()
    }
}

impl From<&RegisteredFunction> for String {
    fn from(function: &RegisteredFunction) -> Self {
        function.to_string()
    }
}

cloud_terrastodon_registry::register_thing!(RegisteredFunction);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiscoveryPlanBuildRequest;
    use cloud_terrastodon_registry::RuntimeValue;

    #[test]
    fn reflected_function_text_preserves_the_registered_executor_identity() -> eyre::Result<()> {
        let function = RegisteredFunction::from_function(
            DiscoveryPlanBuildRequest::REGISTERED_EXECUTABLE_REQUEST,
        );
        let value = RuntimeValue::from_text(RegisteredFunction::SHAPE, &function.to_string())?;
        assert_eq!(*value.peek().get::<RegisteredFunction>()?, function);
        assert!(
            "missing registration"
                .parse::<RegisteredFunction>()
                .is_err()
        );
        Ok(())
    }
}
