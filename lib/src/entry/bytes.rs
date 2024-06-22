use crate::utils::{slice_to_array, slice_to_array_mut};
use std::ops::RangeBounds;
pub use variant::Instance as Variant;
mod private {
    pub trait Instance {}
}
// pub mod wrap;

pub mod variant;

pub type Const<'a> = Value<variant::Const<'a>>;
pub type Mut<'a> = Value<variant::Mut<'a>>;
pub type Owned = Value<variant::Owned>;

#[derive(Clone)]
pub struct Value<V: Variant>(V::Inner);

impl<V: Variant> Value<V> {
    pub(crate) fn new(value: V::Inner) -> Self {
        Self(value)
    }

    pub unsafe fn detach<'b>(self) -> Value<V::Ref<'b>>
    where
        V: variant::Ref,
    {
        Value(V::detach(self.0))
    }

    pub unsafe fn index_range(self, at: usize, len: usize) -> Value<V> {
        Value(V::index_range(self.0, at, len))
    }

    // fn into_owned(self) -> Value<{ O::LEN }, variant::Owned> {
    //     O::into_owned(self)
    // }
}

impl<'a> Const<'a> {
    pub fn rb_const<'b>(&'b self) -> Const<'b> {
        Self::new(&*self.0)
    }
}

impl<'a> Mut<'a> {
    pub fn rb_mut<'b>(&'b mut self) -> Mut<'b> where 'a: 'b {
        Self::new(&mut *self.0)
    }
}

impl<'a> Const<'a> {
    pub fn slice(&self) -> &[u8] {
        self.0
    }

    pub unsafe fn array<const L: usize>(&self) -> &[u8; L] {
        slice_to_array(self.slice())
    }
}

impl<'a> Mut<'a> {
    pub fn slice_mut(&mut self) -> &mut [u8] {
        self.0
    }

    pub unsafe fn array_mut<const L: usize>(&mut self) -> &mut [u8; L] {
        slice_to_array_mut(self.slice_mut())
    }

    pub fn fill(&mut self, value: u8) {
        self.slice_mut().fill(value)
    }

    pub fn fill_with(&mut self, value: impl FnMut() -> u8) {
        self.slice_mut().fill_with(value)
    }

    pub fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.slice_mut().copy_within(src, dest)
    }

    pub fn copy_from(&mut self, src: &Const<'_>) {
        self.slice_mut().copy_from_slice(src.slice())
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.slice_mut().copy_from_slice(slice);
    }

    pub fn swap(&mut self, with: &mut Mut<'_>) {
        self.slice_mut().swap_with_slice(with.slice_mut())
    }
}
