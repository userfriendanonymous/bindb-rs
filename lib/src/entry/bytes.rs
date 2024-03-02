use crate::utils::{slice_to_array, slice_to_array_mut};
use std::ops::{Index, IndexMut, Range, RangeBounds};
pub use variant::Instance as Variant;
mod private {
    pub trait Instance {}
}

pub mod variant;

pub struct Value<V: Variant, const L: usize>(V::Value<L>);

impl<V: Variant, const L: usize> Value<V, L> {
    pub(crate) fn new(value: V::Value<L>) -> Self {
        Self(value)
    }

    unsafe fn detach<'b>(self) -> variant::ValueOf<V::Ref<'b>, L>
    where
        V: variant::Ref,
    {
        V::detach(self.0)
    }

    // fn into_owned(self) -> Value<{ O::LEN }, variant::Owned> {
    //     O::into_owned(self)
    // }
}

impl<V: variant::AsConst, const L: usize> Value<V, L> {
    fn slice(&self) -> &[u8] {
        V::as_const(&self.0)
    }

    fn as_array(self) -> [u8; L] {
        *unsafe { slice_to_array(self.slice()) }
    }
}

impl<V: variant::AsMut, const L: usize> Value<V, L> {
    fn slice_mut(&mut self) -> &mut [u8] {
        V::as_mut(&mut self.0)
    }

    fn fill(&mut self, value: u8) {
        self.slice_mut().fill(value)
    }

    fn fill_with(&mut self, value: impl FnMut() -> u8) {
        self.slice_mut().fill_with(value)
    }

    fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.slice_mut().copy_within(src, dest)
    }

    fn copy_from<SO: variant::AsConst>(&mut self, src: &Value<L, SO>) {
        self.slice_mut().copy_from_slice(src.slice())
    }

    fn swap<SO: variant::AsMut>(&mut self, with: &mut Value<L, SO>) {
        self.slice_mut().swap_with_slice(with.slice_mut())
    }
}
