use std::{marker::PhantomData, ops::{Index, IndexMut, Range, RangeBounds}};
use crate::utils::{slice_to_array, slice_to_array_mut};
use super::Codable;

pub struct Ref<'a, T>(&'a [u8], PhantomData<T>);

impl<'a, T: Codable> PartialEq for Ref<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T: Codable> Eq for Ref<'a, T> {}

impl<'a, T: Codable> Clone for Ref<'a, T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<'a, T: Codable> From<&'a [u8; T::SIZE]> for Ref<'a, T> {
    fn from(value: &'a [u8; T::SIZE]) -> Self {
        Self::new(&*value)
    }
}

impl<'a, T: Codable> Ref<'a, T> {
    const fn new(data: &'a [u8]) -> Self {
        Self(data, PhantomData)
    }

    pub const fn as_array(&self) -> &[u8; T::SIZE] {
        unsafe { slice_to_array(self.0) }
    }

    pub const fn index_to<O: Codable>(&self, at: usize) -> Ref<'a, O> {
        Ref::new(&self.0[at .. at + O::SIZE])
    }

    pub const fn into_owned(&self) -> Owned<T> where [(); T::SIZE]: {
        Owned(unsafe { *slice_to_array(self.0) })
    }
}

impl<'a, T: Codable> Index<usize> for Ref<'a, T> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a, T: Codable> Index<Range<usize>> for Ref<'a, T> {
    type Output = [u8];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[index]
    }
}

pub struct Mut<'a, T>(&'a mut [u8], PhantomData<T>);

impl<'a, T: Codable> PartialEq for Mut<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T: Codable> Eq for Mut<'a, T> {}

impl<'a, T: Codable> Clone for Mut<'a, T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<'a, T: Codable> From<&'a mut [u8; T::SIZE]> for Mut<'a, T> {
    fn from(value: &'a mut [u8; T::SIZE]) -> Self {
        Self::new(&mut *value)
    }
}

impl<'a, T: Codable> Mut<'a, T> {
    const fn new(data: &'a mut [u8]) -> Self {
        Self(data, PhantomData)
    }

    pub const fn fill(&mut self, value: u8) {
        self.0.fill(value)
    }

    pub const fn fill_with(&mut self, value: impl FnMut() -> u8) {
        self.0.fill_with(value)
    }

    pub const fn as_array(&self) -> &[u8; T::SIZE] {
        unsafe { slice_to_array(self.0) }
    }

    pub const fn to_ref(&self) -> Ref<'a, T> {
        Ref::new(&*self.0)
    }

    pub const fn index_to<O: Codable>(&self, at: usize) -> Mut<'a, O> {
        Mut::new(&mut self.0[at .. at + O::SIZE])
    }

    pub const fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.0.copy_within(src, dest)
    }

    pub const fn copy_from(&mut self, src: &Ref<'a, T>) {
        self.0.copy_from_slice(src.0)
    }

    pub const fn swap(&mut self, src: &mut Mut<'a, T>) {
        self.0.swap_with_slice(src.0)
    }

    pub unsafe fn detach<'b>(&'a mut self) -> Mut<'b, T> {
        Mut::new(&mut *(self.0 as *mut _))
    }

    pub const fn into_owned(&mut self) -> Owned<T> where [(); T::SIZE]: {
        Owned(unsafe { *slice_to_array_mut(self.0) })
    }
}

impl<'a, T: Codable> Index<usize> for Mut<'a, T> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a, T: Codable> Index<Range<usize>> for Mut<'a, T> {
    type Output = [u8];
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a, T: Codable> IndexMut<usize> for Mut<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<'a, T: Codable> IndexMut<Range<usize>> for Mut<'a, T> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub struct Owned<T: Codable>([u8; T::SIZE]) where [(); T::SIZE]:;

impl<'a, T: Codable> PartialEq for Owned<T> where [(); T::SIZE]: {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T: Codable> Eq for Owned<T> where [(); T::SIZE]: {}

impl<'a, T: Codable> Clone for Owned<T> where [(); T::SIZE]: {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<T: Codable> Owned<T> where [(); T::SIZE]: {
    const fn new(data: [u8; T::SIZE]) -> Self {
        Self(data)
    }

    pub const fn filled_with(v: u8) -> Self {
        Self([v; T::SIZE])
    }

    pub const fn to_ref<'a>(&'a self) -> Ref<'a, T> {
        Ref::new(&self.0)
    }

    pub const fn to_mut<'a>(&'a mut self) -> Mut<'a, T> {
        Mut::new(&mut self.0)
    }
}