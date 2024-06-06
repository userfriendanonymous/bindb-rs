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

impl<'a> Value<variant::Const<'a>> {
    pub fn rb_const(&self) -> Self {
        Self::new(self.slice())
    }
}

impl<'a> Value<variant::Mut<'a>> {
    pub fn rb_mut(&mut self) -> Self {
        Self::new(self.slice_mut())
    }
}

impl<V: variant::AsConst> Value<V> {
    pub fn slice(&self) -> &[u8] {
        V::as_const(&self.0)
    }

    pub unsafe fn array<const L: usize>(&self) -> &[u8; L] {
        slice_to_array(self.slice())
    }

    pub fn as_const(&self) -> Const<'_> {
        Value(self.slice())
    }
}

impl<V: variant::AsMut> Value<V> {
    pub fn slice_mut(&mut self) -> &mut [u8] {
        V::as_mut(&mut self.0)
    }

    pub unsafe fn array_mut<const L: usize>(&self) -> &mut [u8; L] {
        slice_to_array_mut(self.slice_mut())
    }

    pub fn as_mut(&mut self) -> Mut<'_> {
        Value(self.slice_mut())
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

    pub fn copy_from<SO: variant::AsConst>(&mut self, src: &Value<SO>) {
        self.slice_mut().copy_from_slice(src.slice())
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.slice_mut().copy_from_slice(slice);
    }

    pub fn swap<SO: variant::AsMut>(&mut self, with: &mut Value<SO>) {
        self.slice_mut().swap_with_slice(with.slice_mut())
    }
}
