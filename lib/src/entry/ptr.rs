use std::{ops::{Deref, DerefMut, Index, IndexMut, RangeBounds}, slice::SliceIndex};

use crate::{private::Sealed, utils::{slice_to_array, slice_to_array_mut}};

pub trait Instance
    : Clone + Copy + Sized
    + Index<usize, Output = u8> + Deref<Target = [u8]>
    + Sealed
{
    // Doesn't check if range is out of bounds.
    unsafe fn index_range(self, at: usize, len: usize) -> Self;
    fn to_const(self) -> Const;
}

#[derive(Clone, Copy)]
pub struct Const {
    ptr: *const u8,
    len: usize,
}

impl Instance for Const {
    unsafe fn index_range(self, at: usize, len: usize) -> Self {
        Self {
            ptr: self.slice().get_unchecked(at .. at + len).as_ptr(),
            len,
        }
    }

    fn to_const(self) -> Const {
        self
    }
}

impl Sealed for Const {}

impl<'a> From<Const> for &'a [u8] {
    fn from(value: Const) -> Self {
        value.slice()
    }
}

impl Deref for Const {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.slice()
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

    pub fn decode<T: super::Codable>(self) -> T {
        T::decode(T::buf(self))
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
    unsafe fn index_range(self, at: usize, len: usize) -> Self {
        Self {
            ptr: self.slice().get_unchecked_mut(at .. at + len).as_mut_ptr(),
            len
        }
    }

    fn to_const(self) -> Const {
        unsafe { Const::new(self.ptr as *const _, self.len) }
    }
}

impl Deref for Mut {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &*self.slice()
    }
}

impl DerefMut for Mut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice()
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

    pub fn encode<T: super::Codable>(self, value: &T) {
        T::encode(value, T::buf(self))
    }

    pub fn slice<'a>(self) -> &'a mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    // Doesn't check if length is correct.
    pub unsafe fn array<'a, const L: usize>(self) -> &'a [u8; L] {
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