use std::marker::PhantomData;
use super::{Codable, Lensable};

pub struct Value<B, T> {
    _marker: PhantomData<(B, T)>,
    offset: usize,
}

impl<B, T> Clone for Value<B, T> {
    fn clone(&self) -> Self {
        Self::new(self.offset)
    }
}

impl<B, T> Copy for Value<B, T> {}

impl<B> Value<B, B> {
    // pub const TO_SELF: Self = Self::to_self();

    pub fn to_self() -> Self {
        Self {
            _marker: PhantomData::default(),
            offset: 0
        }
    }
}

impl<B, T> Value<B, T> {
    fn new(offset: usize) -> Self {
        Self {
            _marker: Default::default(),
            offset
        }
    }

    pub fn chain<OT>(self, other: Value<T, OT>) -> Value<B, OT> {
        Value {
            _marker: Default::default(),
            offset: self.offset + other.offset
        }
    }

    // pub fn unsafe_new(offset: usize) -> Self {
    //     Self::new(offset)
    // }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<B: Codable, T: Codable> Value<B, T> {
    pub fn create<L>(lens: &L) -> Self where B: Lensable<L, To = T> {
        Self::new(<B as Lensable>::offset(lens))
    }
}

pub trait ToValue<B, T> {
    fn to_lens(self) -> Value<B, T>;
}

impl<B, T> ToValue<B, T> for Value<B, T> {
    fn to_lens(self) -> Value<B, T> {
        self
    }
}

impl<L, B: Codable + Lensable<L, To = T>, T: Codable> ToValue<B, T> for &L {
    fn to_lens(self) -> Value<B, T> {
        Self::new(B::offset(self))
    }
}