use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug)]
pub struct Choice<T> {
    pub key: String,
    pub value: T,
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

pub trait IntoChoices<T> {
    fn into_choices(self) -> Vec<Choice<T>>;
}
impl<T, I> IntoChoices<T> for I
where
    I: IntoIterator,
    I::Item: IntoChoice<T>,
{
    fn into_choices(self) -> Vec<Choice<T>> {
        self.into_iter().map(IntoChoice::into_choice).collect()
    }
}

pub trait IntoChoice<T> {
    fn into_choice(self) -> Choice<T>;
}

impl<T> IntoChoice<T> for Choice<T> {
    fn into_choice(self) -> Choice<T> {
        self
    }
}

impl<T> IntoChoice<T> for T
where
    T: Display,
{
    fn into_choice(self) -> Choice<T> {
        Choice::from(self)
    }
}