use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use std::any::Any;
use std::any::type_name;

#[derive(Debug, Clone, PartialEq, Eq, Default, arbitrary::Arbitrary, facet::Facet)]
#[facet(transparent)]
pub struct ArbitraryBytes(pub Vec<u8>);

impl ArbitraryBytes {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn replace_remaining(&mut self, remaining: Vec<u8>) {
        self.0 = remaining;
    }
}

pub fn arbitrary_from_bytes<T>(input: &mut ArbitraryBytes) -> eyre::Result<T>
where
    T: for<'a> Arbitrary<'a> + Any + Send + 'static,
{
    let mut unstructured = Unstructured::new(input.as_slice());
    let output = T::arbitrary(&mut unstructured)?;
    input.replace_remaining(unstructured.take_rest().to_vec());
    Ok(output)
}

pub fn invoke_arbitrary_mut<T>(input: &mut (dyn Any + Send)) -> eyre::Result<Box<dyn Any + Send>>
where
    T: for<'a> Arbitrary<'a> + Any + Send + 'static,
{
    let bytes = input.downcast_mut::<ArbitraryBytes>().ok_or_else(|| {
        eyre::eyre!(
            "mutable invocation expected input type {}, but received a different erased value",
            type_name::<ArbitraryBytes>()
        )
    })?;
    Ok(Box::new(arbitrary_from_bytes::<T>(bytes)?) as Box<dyn Any + Send>)
}

crate::register_thing!(ArbitraryBytes);
