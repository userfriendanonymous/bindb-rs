use std::marker::PhantomData;
use crate::{entry::{self}, Entry};

pub const fn identity<E>() -> Identity<E> {
    Identity::SELF
}

pub trait Instance<In: Entry, Out: Entry> {
    fn apply<P: entry::Ptr>(self, buf: entry::Buf<In, P>) -> entry::Buf<Out, P>;
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

impl<E: Entry> Instance<E, E> for Identity<E> {
    fn apply<P: entry::Ptr>(self, buf: entry::Buf<E, P>) -> entry::Buf<E, P> {
        buf
    }
}
