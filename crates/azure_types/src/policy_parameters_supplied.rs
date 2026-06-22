use facet_json::RawJson;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq, Default, facet::Facet)]
#[facet(transparent)]
pub struct AzurePolicyDefinitionParametersSupplied(
    pub HashMap<String, AzurePolicyDefinitionParametersSuppliedValue>,
);

impl Deref for AzurePolicyDefinitionParametersSupplied {
    type Target = HashMap<String, AzurePolicyDefinitionParametersSuppliedValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, facet::Facet)]
pub struct AzurePolicyDefinitionParametersSuppliedValue {
    pub value: RawJson<'static>,
}

impl From<&str> for AzurePolicyDefinitionParametersSuppliedValue {
    fn from(value: &str) -> Self {
        AzurePolicyDefinitionParametersSuppliedValue {
            value: RawJson::from_owned(
                facet_json::to_string(value)
                    .expect("serializing a string policy parameter should not fail"),
            ),
        }
    }
}

impl From<String> for AzurePolicyDefinitionParametersSuppliedValue {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl<K, V> FromIterator<(K, V)> for AzurePolicyDefinitionParametersSupplied
where
    K: Into<String>,
    V: Into<AzurePolicyDefinitionParametersSuppliedValue>,
{
    /// Constructs a `AzurePolicyDefinitionParametersSupplied` from an iterator of key-value pairs.
    ///
    /// If the iterator produces any pairs with equal keys,
    /// all but one of the corresponding values will be dropped.
    fn from_iter<T: IntoIterator<Item = (K, V)>>(
        iter: T,
    ) -> AzurePolicyDefinitionParametersSupplied {
        let mut map = HashMap::with_hasher(Default::default());
        map.extend(iter.into_iter().map(|(k, v)| (k.into(), v.into())));
        AzurePolicyDefinitionParametersSupplied(map)
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for AzurePolicyDefinitionParametersSupplied
where
    K: Into<String>,
    V: Into<AzurePolicyDefinitionParametersSuppliedValue>,
{
    fn from(arr: [(K, V); N]) -> Self {
        Self::from_iter(arr)
    }
}
