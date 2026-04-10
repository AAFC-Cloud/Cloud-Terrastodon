use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct AzurePolicyDefinitionParametersSupplied(
    pub HashMap<String, AzurePolicyDefinitionParametersSuppliedValue>,
);

impl Deref for AzurePolicyDefinitionParametersSupplied {
    type Target = HashMap<String, AzurePolicyDefinitionParametersSuppliedValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AzurePolicyDefinitionParametersSuppliedValue {
    pub value: Value,
}
impl<V> From<V> for AzurePolicyDefinitionParametersSuppliedValue
where
    V: Into<Value>,
{
    fn from(value: V) -> Self {
        AzurePolicyDefinitionParametersSuppliedValue {
            value: value.into(),
        }
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
