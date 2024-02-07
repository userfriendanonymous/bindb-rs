use std::marker::PhantomData;
use crate::Lens;

pub mod basic;

pub type Cached<T> = Buf<T>;

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

impl<'a, T> private::Write for BufRef<'a, T> {}

impl<'a, T: Instance> BufRef<'a, T> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self(data, Default::default())
    }

    pub fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(self.0)
    }

    pub fn to<I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufRef<'a, I> {
        BufRef(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn to_owned(&self) -> Buf<T> {
        Buf(self.0.to_vec(), self.1)
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

impl<'a, T: Instance> BufMut<'a, T> {
    pub(crate) fn new(data: &'a mut [u8]) -> Self {
        Self(data, Default::default())
    }
    
    pub fn to<I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufMut<'a, I> {
        BufMut(&mut self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn ref_to<I: Instance>(&'a self, lens: Lens<T, I>) -> BufRef<'a, I> {
        BufRef(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(self.0)
    }

    pub fn set(&mut self, to: &impl Write<T>) {
        to.write(self.0);
    }

    pub fn set_within<I: Instance>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
        self.0.copy_within(from.offset() .. from.offset() + I::size(), to.offset());
    }

    /// Returns modified data.
    pub fn modify(&mut self, f: impl Fn(T) -> T) -> Result<T, <T as Instance>::DecodeError> {
        let modified = f(self.decode()?);
        self.set(&modified);
        Ok(modified)
    }

    pub fn to_owned(&self) -> Buf<T> {
        Buf(self.0.to_vec(), self.1)
    }

    pub fn to_ref(&'a self) -> BufRef<'a, T> {
        BufRef(&self.0, self.1)
    }
}

pub struct Buf<T>(Vec<u8>, PhantomData<T>);

impl<T> private::Write for Buf<T> {}

impl<T> PartialEq for Buf<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Buf<T> {}

impl<T: Instance> Buf<T> {
    pub fn encode(value: &T) -> Self {
        let mut bytes = Vec::new();
        value.encode(&mut bytes);
        Self(bytes, Default::default())
    }

    pub fn mut_to<'a, I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufMut<'a, I> {
        BufMut(&mut self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn ref_to<'a, I: Instance>(&'a mut self, lens: Lens<T, I>) -> BufRef<'a, I> {
        BufRef(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    }

    pub fn decode(&self) -> Result<T, <T as Instance>::DecodeError> {
        T::decode(&self.0)
    }

    pub fn set(&mut self, to: &impl Write<T>) {
        to.write(&mut self.0);
    }

    pub fn set_within<I: Instance>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
        self.0.copy_within(from.offset() .. from.offset() + I::size(), to.offset());
    }

    pub fn to_owned(&self) -> Buf<T> {
        Buf(self.0.to_vec(), self.1)
    }

    pub fn to_ref<'a>(&'a self) -> BufRef<'a, T> {
        BufRef(&self.0, self.1)
    }

    pub fn to_mut<'a>(&'a mut self) -> BufMut<'a, T> {
        BufMut(&mut self.0, self.1)
    }
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

impl<T> Write<T> for Buf<T> {
    fn write(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.0);
    }
}

impl<T: Instance> Write<T> for T {
    fn write(&self, bytes: &mut [u8]) {
        self.encode(bytes);
    }
}
