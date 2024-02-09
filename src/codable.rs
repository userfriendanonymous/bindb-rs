use std::{marker::PhantomData, ptr};
use crate::Lens;

pub mod basic;

pub type Cached<T> = BufOwned<T>;

mod private {
    pub trait Write {}
}

pub trait Instance {
    type DecodeError;

    fn encode(&self, bytes: &mut [u8]);
    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized;

    fn size() -> usize;
}

impl<T: Instance> private::Write for T {}

pub struct BufRef<'a, T>(&'a [u8], PhantomData<T>);

impl<'a, T> PartialEq for BufRef<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T> Eq for BufRef<'a, T> {}

impl<'a, T> Clone for BufRef<'a, T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<'a, T> private::Write for BufRef<'a, T> {}

impl<'b, T> Buf<T> for BufRef<'b, T> {
    fn to_ref<'a>(&'a self) -> BufRef<'a, T> {
        self.clone()
    }
}

impl<'a, T> BufRef<'a, T> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self(data, Default::default())
    }
}

impl<'a, T: Instance> BufRef<'a, T> {
    pub fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(self.0)
    }

    pub fn to<I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufRef<'a, I> {
        BufRef(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn to_owned(&self) -> BufOwned<T> {
        BufOwned(self.0.to_vec(), self.1)
    }
}

pub struct BufMut<'a, T>(&'a mut [u8], PhantomData<T>);

impl<'a, T> private::Write for BufMut<'a, T> {}

impl<'a, T> PartialEq for BufMut<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T> Eq for BufMut<'a, T> {}

impl<'b, T> Buf<T> for BufMut<'b, T> {
    fn to_ref<'a>(&'a self) -> BufRef<'a, T> {
        BufRef(&self.0, self.1)
    }
}

impl<'a, T> BufMut<'a, T> {
    pub(crate) fn new(data: &'a mut [u8]) -> Self {
        Self(data, Default::default())
    }

    /// Changes lifetime.
    pub unsafe fn detach<'b>(&'a mut self) -> BufMut<'b, T> {
        BufMut::new(&mut *(self.0 as *mut _))
    }

    pub fn to_mut_ptr(&mut self) -> BufMutPtr<T> {
        BufMutPtr::new(self.0 as *mut _)
    }

    pub fn to_owned(&self) -> BufOwned<T> {
        BufOwned(self.0.to_vec(), self.1)
    }

    pub fn swap(&mut self, other: &mut Self) {
        self.0.swap_with_slice(other.0);
    }

    // Parts must not overlap.
    pub fn swap_within<I: Instance>(&mut self, a: Lens<T, I>, b: Lens<T, I>) {
        let size = I::size();
        if a.offset().abs_diff(b.offset()) < size {
            panic!("Overlapping lens!");
        }
        unsafe {
            ptr::swap_nonoverlapping(
                (&mut self.0[a.offset() .. a.offset() + size]).as_mut_ptr(),
                (&mut self.0[b.offset() .. b.offset() + size]).as_mut_ptr(),
                I::size()
            );
        }
    }

    pub fn set(&mut self, to: &impl Write<T>) {
        to.write(self.0);
    }

    pub fn copy_within<I: Instance>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
        self.0.copy_within(from.offset() .. from.offset() + I::size(), to.offset());
    }

    pub fn to<I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufMut<'a, I> {
        BufMut(&mut self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn ref_to<I: Instance>(&'a self, lens: Lens<T, I>) -> BufRef<'a, I> {
        BufRef(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }
}

impl<'a, T: Instance> BufMut<'a, T> {
    pub fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(self.0)
    }

    /// Returns modified data.
    pub fn modify(&mut self, f: impl Fn(T) -> T) -> Result<T, <T as Instance>::DecodeError> {
        let modified = f(self.decode()?);
        self.set(&modified);
        Ok(modified)
    }
}

pub struct BufMutPtr<T>(*mut [u8], PhantomData<T>);

impl<T> BufMutPtr<T> {
    pub(crate) fn new(data: *mut [u8]) -> Self {
        Self(data, Default::default())
    }

    pub unsafe fn set(&mut self, to: &impl Write<T>) {
        to.write(&mut *self.0);
    }

    pub unsafe fn set_within<I: Instance>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
        (&mut *self.0).copy_within(from.offset() .. from.offset() + I::size(), to.offset());
    }

    /// Must not overlap.
    pub unsafe fn swap(&mut self, other: &mut BufMutPtr<T>) {
        (&mut *self.0).swap_with_slice(&mut *other.0);
    }

    pub unsafe fn to<I: Instance>(&mut self, lens: Lens<T, I>) -> BufMutPtr<I> {
        BufMutPtr((&mut (&mut *self.0)[lens.offset() .. lens.offset() + I::size()]) as *mut _, Default::default())
    }

    pub unsafe fn to_owned(&self) -> BufOwned<T> {
        BufOwned((&*self.0).to_vec(), self.1)
    }

    // Removed.
    // pub unsafe fn to_ref<'a>(&self) -> BufRef<'a, T> {
    //     BufRef::new(&*self.0)
    // }

    // pub unsafe fn to_mut<'a>(&self) -> BufMut<'a, T> {
    //     BufMut::new(&mut *self.0)
    // }
}

impl<T: Instance> BufMutPtr<T> {
    pub unsafe fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(&*self.0)
    }

    /// Returns modified data.
    pub unsafe fn modify(&mut self, f: impl Fn(T) -> T) -> Result<T, <T as Instance>::DecodeError> {
        let modified = f(self.decode()?);
        self.set(&modified);
        Ok(modified)
    }
}

pub struct BufOwned<T>(Vec<u8>, PhantomData<T>);

impl<T> private::Write for BufOwned<T> {}

impl<T> PartialEq for BufOwned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for BufOwned<T> {}

impl<T> Buf<T> for BufOwned<T> {
    fn to_ref<'a>(&'a self) -> BufRef<'a, T> {
        BufRef(&self.0, self.1)
    }
}

impl<T> BufOwned<T> {
    pub fn to_mut<'a>(&'a mut self) -> BufMut<'a, T> {
        BufMut(&mut self.0, self.1)
    }
}

impl<T: Instance> BufOwned<T> {
    pub fn encode(value: &T) -> Self {
        let mut bytes = vec![0; T::size()];
        value.encode(&mut bytes);
        Self(bytes, Default::default())
    }

    pub fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(&self.0)
    }

    // pub fn mut_to<'a, I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufMut<'a, I> {
    //     BufMut(&mut self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    // }

    // pub fn ref_to<'a, I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufRef<'a, I> {
    //     BufRef(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    // }

    // pub fn set(&mut self, to: &impl Write<T>) {
    //     to.write(&mut self.0);
    // }

    // pub fn set_within<I: Instance>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
    //     self.0.copy_within(from.offset() .. from.offset() + I::size(), to.offset());
    // }

    // pub fn to_owned(&self) -> Buf<T> {
    //     Buf(self.0.to_vec(), self.1)
    // }
}

pub trait Write<T>: private::Write {
    fn write(&self, bytes: &mut [u8]);
}

impl<'a, T> Write<T> for BufRef<'a, T> {
    fn write(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.0);
    }
}

impl<'a, T> Write<T> for BufMut<'a, T> {
    fn write(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.0);
    }
}

impl<T> Write<T> for BufOwned<T> {
    fn write(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.0);
    }
}

impl<T: Instance> Write<T> for T {
    fn write(&self, bytes: &mut [u8]) {
        self.encode(bytes);
    }
}

pub trait Buf<T>: Write<T> {
    fn to_ref<'a>(&'a self) -> BufRef<'a, T>;
}

pub trait AsBuf<'a, T> {
    type Buf: Buf<T> + 'a;
    fn as_buf(&'a self) -> Self::Buf;
}

impl<'a, T: 'a> AsBuf<'a, T> for BufRef<'a, T> {
    type Buf = Self;
    fn as_buf(&'a self) -> Self::Buf {
        self.clone()
    }
}

impl<'a, T: 'a> AsBuf<'a, T> for BufMut<'a, T> {
    type Buf = BufRef<'a, T>;
    fn as_buf(&'a self) -> Self::Buf {
        self.to_ref()
    }
}

impl<'a, T: 'a> AsBuf<'a, T> for BufOwned<T> {
    type Buf = BufRef<'a, T>;
    fn as_buf(&'a self) -> Self::Buf {
        self.to_ref()
    }
}

impl<'a, T: Instance + 'a> AsBuf<'a, T> for T {
    type Buf = BufOwned<T>;
    fn as_buf(&'a self) -> Self::Buf {
        BufOwned::encode(self)
    }
}