use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct Value<B, T> {
    _marker: PhantomData<(B, T)>,
    offset: usize,
}

impl<B> Value<B, B> {
    pub const TO_SELF: Self = Self::to_self();

    pub const fn to_self() -> Self {
        Self {
            _marker: Default::default(),
            offset: 0
        }
    }
}

impl<B, T> Value<B, T> {
    pub fn chain<OT>(self, other: Value<T, OT>) -> Value<B, OT> {
        Value {
            _marker: Default::default(),
            offset: self.offset + other.offset
        }
    }

    pub fn unsafe_new(offset: usize) -> Self {
        Self {
            _marker: Default::default(),
            offset
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}