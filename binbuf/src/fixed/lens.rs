use std::marker::PhantomData;
use super::{Instance as Entry, Buf};

pub const fn identity<E>() -> Identity<E> {
    Identity::SELF
}

pub trait Instance<In: Entry, Out: Entry> {
    fn apply<P: super::Ptr>(self, buf: Buf<In, P>) -> Buf<Out, P>;
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
    fn apply<P: super::Ptr>(self, buf: Buf<E, P>) -> Buf<E, P> {
        buf
    }
}
