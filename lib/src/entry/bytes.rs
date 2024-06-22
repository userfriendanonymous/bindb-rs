use crate::utils::{slice_to_array, slice_to_array_mut};
use std::ops::RangeBounds;
pub use variant::Instance as Variant;
mod private {
    pub trait Instance {}
}
// pub mod wrap;

pub mod variant;

pub type Const = Value<variant::Const>;
pub type Mut = Value<variant::Mut>;
pub type Owned = Value<variant::Owned>;

#[derive(Clone, Copy)]
pub struct Value<V: Variant>(V::Data);

impl<V: Variant> Value<V> {
    pub(crate) unsafe fn new(value: V::Data) -> Self {
        Self(value)
    }

    // pub unsafe fn detach<'b>(self) -> Value<V::Ref<'b>>
    // where
    //     V: variant::Ref,
    // {
    //     Value(V::detach(self.0))
    // }

    pub unsafe fn index_range(self, at: usize, len: usize) -> Value<V> {
        Value(V::index_range(self.0, at, len))
    }

    // fn into_owned(self) -> Value<{ O::LEN }, variant::Owned> {
    //     O::into_owned(self)
    // }
}

impl<V: variant::AsConst> Value<V> {
    fn as_const(&self) -> Const {
        Value(V::as_const(&self.0))
    }
}

impl<V: variant::AsMut> Value<V> {
    fn as_mut(&mut self) -> Mut {
        Value(V::as_mut(&mut self.0))
    }
}

// impl Const {
//     pub fn rb_const<'b>(&'b self) -> Const<'b> {
//         Self::new(&*self.0)
//     }
// }

// impl<'a> Mut<'a> {
//     pub fn rb_mut<'b>(&'b mut self) -> Mut<'b> where 'a: 'b {
//         Self::new(&mut *self.0)
//     }
// }

impl Const {
    pub fn slice(self) -> &[u8] {
        unsafe { self.0.as_ref() }
    }

    pub unsafe fn array<const L: usize>(self) -> &[u8; L] {
        slice_to_array(self.slice())
    }
}

impl Mut {
    pub fn slice_mut(self) -> &mut [u8] {
        unsafe { self.0.as_mut() }
    }

    pub unsafe fn array_mut<const L: usize>(self) -> &mut [u8; L] {
        slice_to_array_mut(self.slice_mut())
    }

    pub fn fill(self, value: u8) {
        self.slice_mut().fill(value)
    }

    pub fn fill_with(self, value: impl FnMut() -> u8) {
        self.slice_mut().fill_with(value)
    }

    pub fn copy_within<R: RangeBounds<usize>>(self, src: R, dest: usize) {
        self.slice_mut().copy_within(src, dest)
    }

    pub fn copy_from(self, src: &Const<'_>) {
        self.slice_mut().copy_from_slice(src.slice())
    }

    pub fn copy_from_slice(self, slice: &[u8]) {
        self.slice_mut().copy_from_slice(slice);
    }

    pub fn swap(self, with: Mut<'_>) {
        self.slice_mut().swap_with_slice(with.slice_mut())
    }
}
