use crate::utils::slice_to_array;
use std::ops::RangeBounds;
pub use variant::Instance as Variant;
mod private {
    pub trait Instance {}
}
// pub mod wrap;

pub mod variant;

pub type Const<'a, const L: usize> = Value<variant::Const<'a>, L>;
pub type Mut<'a, const L: usize> = Value<variant::Mut<'a>, L>;
pub type Owned<const L: usize> = Value<variant::Owned, L>;

pub struct Value<V: Variant, const L: usize>(V::Inner<L>);

impl<V: Variant, const L: usize> Value<V, L> {
    pub(crate) fn new(value: V::Inner<L>) -> Self {
        Self(value)
    }

    pub unsafe fn detach<'b>(self) -> Value<V::Ref<'b>, L>
    where
        V: variant::Ref,
    {
        Value(V::detach(self.0))
    }

    pub unsafe fn const_index<const OL: usize>(self, at: usize) -> Value<V, OL> {
        Value(V::const_index(self.0, at))
    }

    // fn into_owned(self) -> Value<{ O::LEN }, variant::Owned> {
    //     O::into_owned(self)
    // }
}

impl<'a, const L: usize> Value<variant::Const<'a>, L> {
    pub fn rb_const(&self) -> Self {
        Self::new(self.slice())
    }
}

impl<'a, const L: usize> Value<variant::Mut<'a>, L> {
    pub fn rb_mut(&mut self) -> Self {
        Self::new(self.slice_mut())
    }
}

impl<V: variant::AsConst, const L: usize> Value<V, L> {
    pub fn slice(&self) -> &[u8] {
        V::as_const(&self.0)
    }

    pub fn as_array(self) -> [u8; L] {
        *unsafe { slice_to_array(self.slice()) }
    }

    pub fn as_const(&self) -> Const<'_, L> {
        Value(self.slice())
    }
}

impl<V: variant::AsMut, const L: usize> Value<V, L> {
    pub fn slice_mut(&mut self) -> &mut [u8] {
        V::as_mut(&mut self.0)
    }

    pub fn as_mut(&mut self) -> Mut<'_, L> {
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

    pub fn copy_from<SO: variant::AsConst>(&mut self, src: &Value<SO, L>) {
        self.slice_mut().copy_from_slice(src.slice())
    }

    pub fn swap<SO: variant::AsMut>(&mut self, with: &mut Value<SO, L>) {
        self.slice_mut().swap_with_slice(with.slice_mut())
    }
}
