use std::marker::PhantomData;
use super::Value;

pub struct Empty<B>(Root<B>);

impl<B> Empty<B> {
    pub fn new(root: Root<B>) -> Self {
        Self(root)
    }
}

pub struct Root<B>(PhantomData<B>);

impl<B> Root<B> {
    pub(crate) const VALUE: Self = Self(PhantomData);

    pub fn spawn<T>(&self, offset: usize) -> Value<B, T> {
        Value::new(offset)
    }
}
