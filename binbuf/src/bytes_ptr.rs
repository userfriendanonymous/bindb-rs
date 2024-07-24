use std::ops::{self, Index, IndexMut, RangeBounds};
use crate::{private::Sealed, utils::{slice_to_array, slice_to_array_mut}};

pub trait Instance
    : Clone + Copy + Sized
    + Index<usize, Output = u8>
    + Sealed
{
    // Doesn't check if range is out of bounds.
    unsafe fn range_at(self, at: usize, len: usize) -> Self;
    unsafe fn range(self, start: usize, end: usize) -> Self;
    unsafe fn range_from(self, start: usize) -> Self;
    fn len(self) -> usize;
    fn to_const(self) -> Const;
}

#[derive(Clone, Copy)]
pub struct Const {
    ptr: *const u8,
    len: usize,
}

impl Instance for Const {
    unsafe fn range_at(self, at: usize, len: usize) -> Self {
        Self {
            ptr: self.slice().get_unchecked(at .. at + len).as_ptr(),
            len,
        }
    }

    unsafe fn range(self, start: usize, end: usize) -> Self {
        let slice = self.slice().get_unchecked(start .. end);
        Self { ptr: slice.as_ptr(), len: slice.len() }
    }

    unsafe fn range_from(self, start: usize) -> Self {
        let slice = self.slice().get_unchecked(start ..);
        Self { ptr: slice.as_ptr(), len: slice.len() }
    }

    fn to_const(self) -> Const {
        self
    }
    
    fn len(self) -> usize {
        self.len
    }
}

impl Sealed for Const {}

impl<'a> From<Const> for &'a [u8] {
    fn from(value: Const) -> Self {
        value.slice()
    }
}

impl Index<usize> for Const {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        self.slice().index(index)
    }
}

impl Const {
    pub unsafe fn new(ptr: *const u8, len: usize) -> Self {
        Self { ptr, len }
    }

    // Ensure this ptr is of correct length.
    // pub unsafe fn decode<T: crate::entry::Codable>(self) -> T {
    //     T::decode(T::buf(self))
    // }

    pub unsafe fn cast_to_ref<'a, T>(self) -> &'a T {
        &*self.ptr.cast::<T>()
    }

    pub unsafe fn from_slice(slice: &[u8]) -> Self {
        Self { ptr: slice.as_ptr(), len: slice.len() }
    }

    pub fn slice<'a>(self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    // Doesn't check if length is correct.
    pub unsafe fn array<'a, const L: usize>(self) -> &'a [u8; L] {
        slice_to_array(self.slice())
    }

    pub fn copy_to(self, dst: Mut) {
        dst.copy_from(self)
    }

    // Doesn't check if index is valid.
    // pub unsafe fn get<I: SliceIndex<u8>>(self, index: I) -> &I::Output {
    //     self.slice().get_unchecked(index)
    // }
}

#[derive(Clone, Copy)]
pub struct Mut {
    ptr: *mut u8,
    len: usize,
}

impl Sealed for Mut {}

impl<'a> From<Mut> for &'a [u8] {
    fn from(value: Mut) -> Self {
        &*value.slice()
    }
}

impl<'a> From<Mut> for &'a mut [u8] {
    fn from(value: Mut) -> Self {
        value.slice()
    }
}

impl Instance for Mut {
    unsafe fn range_at(self, at: usize, len: usize) -> Self {
        Self {
            ptr: self.slice().get_unchecked_mut(at .. at + len).as_mut_ptr(),
            len,
        }
    }

    unsafe fn range(self, start: usize, end: usize) -> Self {
        let slice = self.slice().get_unchecked_mut(start .. end);
        Self { ptr: slice.as_mut_ptr(), len: slice.len() }
    }

    unsafe fn range_from(self, start: usize) -> Self {
        let slice = self.slice().get_unchecked_mut(start ..);
        Self { ptr: slice.as_mut_ptr(), len: slice.len() }
    }

    fn to_const(self) -> Const {
        unsafe { Const::new(self.ptr as *const _, self.len) }
    }

    fn len(self) -> usize {
        self.len
    }
}

impl Index<usize> for Mut {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        self.slice().index(index)
    }
}

impl IndexMut<usize> for Mut {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.slice().index_mut(index)
    }
}

impl Mut {
    pub unsafe fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }

    pub unsafe fn from_slice(slice: &mut [u8]) -> Self {
        Self { ptr: slice.as_mut_ptr(), len: slice.len() }
    }

    pub unsafe fn cast_to_ref<'a, T>(self) -> &'a mut T {
        &mut *self.ptr.cast::<T>()
    }

    // Ensure this ptr is of correct length.
    // pub unsafe fn encode<T: entry::Codable>(self, value: &T) {
    //     T::encode(value, T::buf(self))
    // }

    pub fn slice<'a>(self) -> &'a mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    // Doesn't check if length is correct.
    pub unsafe fn array<'a, const L: usize>(self) -> &'a mut [u8; L] {
        slice_to_array_mut(self.slice())
    }

    pub fn fill(self, value: u8) {
        self.slice().fill(value)
    }

    pub fn fill_with(self, value: impl FnMut() -> u8) {
        self.slice().fill_with(value)
    }

    pub fn copy_within<R: RangeBounds<usize>>(self, src: R, dest: usize) {
        self.slice().copy_within(src, dest)
    }

    pub fn copy_from(self, src: Const) {
        self.slice().copy_from_slice(src.slice())
    }

    pub fn copy_from_slice(self, slice: &[u8]) {
        self.slice().copy_from_slice(slice);
    }

    pub fn swap(self, with: Mut) {
        self.slice().swap_with_slice(with.slice())
    }
}