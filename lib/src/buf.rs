use std::{marker::PhantomData, ptr};
use crate::{utils::{slice_to_array, slice_to_array_mut}, Lens};
use super::Codable;

pub mod bytes;

mod private {
    pub trait Write {}
}

impl<T: Codable> private::Write for T {}

pub struct Ref<'a, T: Codable>(bytes::Ref<'a, T>, PhantomData<T>);

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

impl<'a, T: Codable> private::Write for Ref<'a, T> {}

impl<'b, T: Codable> Instance<T> for Ref<'b, T> {
    fn to_ref<'a>(&'a self) -> Ref<'a, T> {
        self.clone()
    }
}

impl<'a, T: Codable> Ref<'a, T> {
    pub(crate) const fn new(data: &'a [u8]) -> Self {
        Self(data, PhantomData)
    }
}

impl<'a, T: Codable> Ref<'a, T> {
    pub fn decode(&self) -> T {
        T::decode(&self.0)
    }

    pub fn to<I: Codable>(&'a mut self, lens: Lens<T, I>) -> Ref<'a, I> {
        Ref::new(unsafe { slice_to_array(&self.0[lens.offset() .. lens.offset() + I::SIZE]) })
    }

    pub fn to_owned(&self) -> Owned<T> where [(); T::SIZE]: {
        Owned::new(self.0.clone())
    }
}

pub struct Mut<'a, T: Codable>(bytes::Mut<'a, T>, PhantomData<T>);

impl<'a, T: Codable> private::Write for Mut<'a, T> {}

impl<'a, T: Codable> PartialEq for Mut<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T: Codable> Eq for Mut<'a, T> {}

impl<'b, T: Codable> Instance<T> for Mut<'b, T> {
    fn to_ref<'a>(&'a self) -> Ref<'a, T> {
        Ref(&self.0, self.1)
    }
}

impl<'a, T: Codable> Mut<'a, T> {
    pub(crate) const fn new(data: &'a mut [u8]) -> Self {
        Self(data, PhantomData)
    }

    /// Changes lifetime.
    pub unsafe fn detach<'b>(&'a mut self) -> Mut<'b, T> {
        Mut::new(self.0.detach())
    }

    // pub fn to_mut_ptr(&mut self) -> MutPtr<T> {
    //     MutPtr::new(self.0 as *mut _)
    // }

    pub fn to_owned(&self) -> Owned<T> where [(); T::SIZE]: {
        Owned::new(self.0.clone())
    }

    pub fn swap(&mut self, other: &mut Self) {
        self.0.swap_with_slice(other.0);
    }

    // Parts must not overlap.
    pub fn swap_within<I: Codable>(&mut self, a: Lens<T, I>, b: Lens<T, I>) {
        if a.offset().abs_diff(b.offset()) < I::SIZE {
            panic!("Overlapping lens!");
        }
        unsafe {
            ptr::swap_nonoverlapping(
                (&mut self.0[a.offset() .. a.offset() + I::SIZE]).as_mut_ptr(),
                (&mut self.0[b.offset() .. b.offset() + I::SIZE]).as_mut_ptr(),
                I::SIZE
            );
        }
    }

    pub fn set(&mut self, to: &impl Write<T>) {
        to.write(&mut self.0);
    }

    pub fn copy_within<I: Codable>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
        self.0.copy_within(from.offset() .. from.offset() + I::SIZE, to.offset());
    }

    pub fn to<I: Codable>(&'a mut self, lens: Lens<T, I>) -> Mut<'a, I> where [(); I::SIZE]: {
        Mut::new(unsafe { slice_to_array_mut(&mut self.0[lens.offset() .. lens.offset() + I::SIZE]) })
    }

    pub fn ref_to<I: Codable>(&'a self, lens: Lens<T, I>) -> Ref<'a, I> where [(); I::SIZE]: {
        Ref::new(unsafe { slice_to_array(&self.0[lens.offset() .. lens.offset() + I::SIZE]) })
    }

    pub fn decode(&self) -> T {
        T::decode(self.0)
    }

    /// Returns modified data.
    pub fn modify(&mut self, f: impl Fn(T) -> T) -> T {
        let modified = f(self.decode());
        self.set(&modified);
        modified
    }
}

// pub struct MutPtr<T: Codable>(*mut [u8; T::SIZE], PhantomData<T>);

// impl<T: Codable> MutPtr<T> {
//     pub(crate) fn new(data: *mut [u8; T::SIZE]) -> Self {
//         Self(data, Default::default())
//     }

//     pub unsafe fn set(&mut self, to: &impl Write<T>) {
//         to.write(&mut *self.0);
//     }

//     pub unsafe fn set_within<I: Codable>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
//         (&mut *self.0).copy_within(from.offset() .. from.offset() + I::SIZE, to.offset());
//     }

//     /// Must not overlap.
//     pub unsafe fn swap(&mut self, other: &mut MutPtr<T>) {
//         (&mut *self.0).swap_with_slice(&mut *other.0);
//     }

//     // pub unsafe fn to<I: Codable>(&mut self, lens: Lens<T, I>) -> MutPtr<I> where [(); I::SIZE]: {
//     //     MutPtr((&mut (&mut *self.0)[lens.offset() .. lens.offset() + I::size()]) as *mut _, Default::default())
//     // }

//     pub unsafe fn to_owned(&self) -> Owned<T> {
//         Owned(*self.0, self.1)
//     }

//     // Removed.
//     // pub unsafe fn to_ref<'a>(&self) -> Ref<'a, T> {
//     //     Ref::new(&*self.0)
//     // }

//     // pub unsafe fn to_mut<'a>(&self) -> Mut<'a, T> {
//     //     Mut::new(&mut *self.0)
//     // }
// }

// impl<T: Codable> MutPtr<T> {
//     pub unsafe fn decode(&self) -> T {
//         T::decode(&*self.0)
//     }

//     /// Returns modified data.
//     pub unsafe fn modify(&mut self, f: impl Fn(T) -> T) -> T {
//         let modified = f(self.decode());
//         self.set(&modified);
//         modified
//     }
// }

pub struct Owned<T: Codable>([u8; T::SIZE], PhantomData<T>) where [(); T::SIZE]:;

impl<T: Codable> private::Write for Owned<T> where [(); T::SIZE]: {}

impl<T: Codable> PartialEq for Owned<T> where [(); T::SIZE]: {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Codable> Eq for Owned<T> where [(); T::SIZE]: {}

impl<T: Codable> Instance<T> for Owned<T> where [(); T::SIZE]: {
    fn to_ref<'a>(&'a self) -> Ref<'a, T> {
        Ref(&self.0, self.1)
    }
}

impl<T: Codable> Owned<T> where [(); T::SIZE]: {
    pub fn to_mut<'a>(&'a mut self) -> Mut<'a, T> {
        Mut(&mut self.0, self.1)
    }
}

impl<T: Codable> Owned<T> where [(); T::SIZE]: {
    pub fn new(bytes: [u8; T::SIZE]) -> Self {
        Self(bytes, Default::default())
    }

    pub fn encode(value: &T) -> Self {
        let mut bytes = [0; T::SIZE];
        value.encode(&mut bytes);
        Self::new(bytes)
    }

    pub fn decode(&self) -> T {
        T::decode(&self.0)
    }

    // pub fn mut_to<'a, I: Codable>(&'a mut self, lens: Lens<T, I>) -> Mut<'a, I> {
    //     Mut(&mut self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    // }

    // pub fn ref_to<'a, I: Codable>(&'a mut self, lens: Lens<T, I>) -> Ref<'a, I> {
    //     Ref(&self.0[lens.offset() .. lens.offset() + I::size()], Default::default())
    // }

    // pub fn set(&mut self, to: &impl Write<T>) {
    //     to.write(&mut self.0);
    // }

    // pub fn set_within<I: Codable>(&mut self, from: Lens<T, I>, to: Lens<T, I>) {
    //     self.0.copy_within(from.offset() .. from.offset() + I::size(), to.offset());
    // }

    // pub fn to_owned(&self) -> Instance<T> {
    //     Instance(self.0.to_vec(), self.1)
    // }
}

pub trait Write<T: Codable>: private::Write {
    fn write(&self, bytes: &mut bytes::Mut<'_, T>);
}

impl<'a, T: Codable> Write<T> for Ref<'a, T> {
    fn write(&self, bytes: &mut bytes::Mut<'_, T>) {
        bytes.copy_from_slice(self.0);
    }
}

impl<'a, T: Codable> Write<T> for Mut<'a, T> {
    fn write(&self, bytes: &mut bytes::Mut<'_, T>) {
        bytes.copy_from_slice(self.0);
    }
}

impl<T: Codable> Write<T> for Owned<T> where [(); T::SIZE]: {
    fn write(&self, bytes: &mut bytes::Mut<'_, T>) {
        bytes.copy_from_slice(&self.0);
    }
}

impl<T: Codable> Write<T> for T {
    fn write(&self, bytes: &mut bytes::Mut<'_, T>) {
        self.encode(bytes);
    }
}

pub trait Instance<T: Codable>: Write<T> {
    fn to_ref<'a>(&'a self) -> Ref<'a, T>;
}

pub trait AsInstance<'a, T: Codable> where [(); T::SIZE]: {
    type Value: Instance<T> + 'a;
    fn as_buf(&'a self) -> Self::Value;
}

impl<'a, T: 'a + Codable> AsInstance<'a, T> for Ref<'a, T> where [(); T::SIZE]: {
    type Value = Self;
    fn as_buf(&'a self) -> Self::Value {
        self.clone()
    }
}

impl<'a, T: 'a + Codable> AsInstance<'a, T> for Mut<'a, T> where [(); T::SIZE]: {
    type Value = Ref<'a, T>;
    fn as_buf(&'a self) -> Self::Value {
        self.to_ref()
    }
}

impl<'a, T: 'a + Codable> AsInstance<'a, T> for Owned<T> where [(); T::SIZE]: {
    type Value = Ref<'a, T>;
    fn as_buf(&'a self) -> Self::Value {
        self.to_ref()
    }
}

impl<'a, T: Codable + 'a> AsInstance<'a, T> for T where [(); T::SIZE]: {
    type Value = Owned<T>;
    fn as_buf(&'a self) -> Self::Value {
        Owned::encode(self)
    }
}