use std::marker::PhantomData;

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

    pub fn unsafe_new(offset: usize) -> Self {
        Self::new(offset)
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}