use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug)]
pub struct Choice<T> {
    pub key: String,
    pub value: T,
}
impl<T> std::fmt::Display for Choice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key)
    }
}
impl<T: Clone> Clone for Choice<T> {
    fn clone(&self) -> Self {
        Choice {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

impl<T> From<T> for Choice<T>
where
    T: Display,
{
    fn from(value: T) -> Self {
        Choice {
            key: value.to_string(),
            value,
        }
    }
}

impl<T> Deref for Choice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> PartialEq for Choice<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}
impl<T> Eq for Choice<T> where T: Eq {}
