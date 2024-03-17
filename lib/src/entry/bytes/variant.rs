use std::marker::PhantomData;

use crate::utils::index_array;

pub type Inner<T: Instance, const L: usize> = T::Inner<L>;

pub trait Instance {
    type Inner<const L: usize>;
    unsafe fn const_index<const L: usize, const OL: usize>(inner: Self::Inner<L>, at: usize) -> Self::Inner<OL>;
}

pub trait Ref: Instance {
    type Ref<'a>: Instance;

    unsafe fn detach<'a, const L: usize>(value: Self::Inner<L>) -> Inner<Self::Ref<'a>, L>;
}

pub trait AsConst: Instance {
    fn as_const<'a, const L: usize>(value: &'a Self::Inner<L>) -> Inner<Const<'a>, L>;
}

pub trait AsMut: AsConst {
    fn as_mut<'a, const L: usize>(value: &'a mut Self::Inner<L>) -> Inner<Mut<'a>, L>;
}

pub struct Const<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Const<'a> {
    type Inner<const L: usize> = &'a [u8];
    unsafe fn const_index<const L: usize, const OL: usize>(inner: Self::Inner<L>, at: usize) -> Self::Inner<OL> {
        inner.get_unchecked(at .. at + OL)
    }
}

impl<'c> Ref for Const<'c> {
    type Ref<'b> = Const<'b>;

    unsafe fn detach<'a, const L: usize>(value: Self::Inner<L>) -> Inner<Self::Ref<'a>, L> {
        &*(value as *const _)
    }
}

impl<'b> AsConst for Const<'b> {
    fn as_const<'a, const L: usize>(value: &'a Self::Inner<L>) -> Inner<Const<'a>, L> {
        value
    }
}

pub struct Mut<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Mut<'a> {
    type Inner<const L: usize> = &'a mut [u8];
    unsafe fn const_index<const L: usize, const OL: usize>(inner: Self::Inner<L>, at: usize) -> Self::Inner<OL> {
        inner.get_unchecked_mut(at .. at + OL)
    }
}

impl<'b> Ref for Mut<'b> {
    type Ref<'a> = Mut<'a>;
    unsafe fn detach<'a, const L: usize>(value: Self::Inner<L>) -> Inner<Self::Ref<'a>, L> {
        &mut *(value as *mut _)
    }
}

impl<'b> AsConst for Mut<'b> {
    fn as_const<'a, const L: usize>(value: &'a Self::Inner<L>) -> Inner<Const<'a>, L> {
        value
    }
}

impl<'b> AsMut for Mut<'b> {
    fn as_mut<'a, const L: usize>(value: &'a mut Self::Inner<L>) -> Inner<Mut<'a>, L> {
        value
    }
}

pub struct Owned;

impl Instance for Owned {
    type Inner<const L: usize> = [u8; L];
    unsafe fn const_index<const L: usize, const OL: usize>(inner: Self::Inner<L>, at: usize) -> Self::Inner<OL> {
        *index_array(&inner, at)
    }
}

impl AsConst for Owned {
    fn as_const<'a, const L: usize>(value: &'a Self::Inner<L>) -> Inner<Const<'a>, L> {
        value
    }
}

impl AsMut for Owned {
    fn as_mut<'a, const L: usize>(value: &'a mut Self::Inner<L>) -> Inner<Mut<'a>, L> {
        value
    }
}
