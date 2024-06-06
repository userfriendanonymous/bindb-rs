use std::marker::PhantomData;
use crate::{entry::{self}, Entry};

pub const fn identity<E>() -> Identity<E> {
    Identity::SELF
}

pub trait Instance {
    type In: Entry;
    type Out: Entry;
    fn apply<BV: entry::bytes::Variant>(self, buf: entry::Buf<Self::In, BV>) -> entry::Buf<Self::Out, BV>;
}

pub struct Identity<E>(PhantomData<E>);

impl<E> Clone for Identity<E> {
    fn clone(&self) -> Self {
        Self::SELF
    }
}

impl<E> Copy for Identity<E> {}

impl<E> Identity<E> {
    pub const SELF: Self = Self(PhantomData);
}

impl<E: Entry> Instance for Identity<E> {
    type In = E;
    type Out = E;
    fn apply<BV: entry::bytes::Variant>(self, buf: entry::Buf<Self::In, BV>) -> entry::Buf<Self::Out, BV> {
        buf
    }
}
