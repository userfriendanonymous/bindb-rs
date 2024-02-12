use std::{marker::PhantomData, ops::{Index, IndexMut, Range, RangeBounds}};
use crate::utils::slice_to_array;
use super::Codable;

pub struct Ref<'a, T>(&'a [u8], PhantomData<T>);

impl<'a, T: Codable> Ref<'a, T> {
    const fn new(data: &'a [u8]) -> Self {
        Self(data, PhantomData)
    }

    pub const fn as_array(&self) -> &[u8; T::SIZE] {
        unsafe { slice_to_array(self.0) }
    }

    pub const fn index_to<O: Codable>(&self, at: usize) -> Ref<'a, O> {
        Self::new(&self.0[at .. at + O::SIZE])
    }

    pub const fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.0.copy_within(src, dest)
    }

    pub const fn copy_from(&mut self, src: &Ref<'a, T>) {
        self.0.copy_from_slice(src.0)
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

impl<'a, T: Codable> Mut<'a, T> {
    const fn new(data: &'a mut [u8]) -> Self {
        Self(data, PhantomData)
    }

    pub const fn as_array(&self) -> &[u8; T::SIZE] {
        unsafe { slice_to_array(self.0) }
    }

    pub const fn as_ref(&self) -> Ref<'a, T> {
        Ref::new(&*self.0)
    }

    pub const fn index_to<O: Codable>(&self, at: usize) -> Mut<'a, O> {
        Self::new(&mut self.0[at .. at + O::SIZE])
    }

    pub const fn copy_within<R: RangeBounds<usize>>(&mut self, src: R, dest: usize) {
        self.0.copy_within(src, dest)
    }

    pub const fn copy_from(&mut self, src: &Ref<'a, T>) {
        self.0.copy_from_slice(src.0)
    }

    pub unsafe fn detach<'b>(&'a mut self) -> Mut<'b, T> {
        Mut::new(&mut *(self.0 as *mut _))
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