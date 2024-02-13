use std::marker::PhantomData;
use super::Codable;

pub struct Value<B, T>(usize, PhantomData<(B, T)>);

impl<B, T> Clone for Value<B, T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<B, T> Copy for Value<B, T> {}

impl<B> Value<B, B> {
    // pub const TO_SELF: Self = Self::to_self();

    pub fn to_self() -> Self {
        Self::new(0)
    }
}

impl<B, T> Value<B, T> {
    fn new(offset: usize) -> Self {
        Self(offset, PhantomData)
    }

    pub fn chain<OT>(self, other: Value<T, OT>) -> Value<B, OT> {
        Value::new(self.0 + other.0)
    }

    // pub fn unsafe_new(offset: usize) -> Self {
    //     Self::new(offset)
    // }

    pub fn offset(&self) -> usize {
        self.0
    }
}

impl<B: Codable, T: Codable> Value<B, T> {
    // pub fn create<L>(lens: &L) -> Self where B: Lensable<L, To = T> {
    //     Self::new(<B as Lensable>::offset(lens))
    // }
}

// pub trait ToValue<B, T> {
//     fn to_lens(self) -> Value<B, T>;
// }

// impl<B, T> ToValue<B, T> for Value<B, T> {
//     fn to_lens(self) -> Value<B, T> {
//         self
//     }
// }

// impl<B: Codable, T: Codable> ToValue<B, T> for &B:: {
//     fn to_lens(self) -> Value<B, T> {
//         Self::new(B::offset(self))
//     }
// }

pub struct RootProducer<B>(PhantomData<B>);

impl<B> RootProducer<B> {
    pub(crate) const VALUE: Self = Self(PhantomData);

    pub fn spawn<T>(&self, offset: usize) -> Value<B, T> {
        Value::new(offset)
    }
}
