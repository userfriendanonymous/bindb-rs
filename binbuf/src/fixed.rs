#![allow(type_alias_bounds)]

use std::cmp::Ordering;

pub use lens::Instance as Lens;
pub use crate::{Entry, bytes_ptr as ptr, entry::buf_to_const};
pub use ptr::Instance as Ptr;

pub mod lens;

pub type Buf<T: Instance, P> = T::Buf<P>;
pub type BufConst<T: Instance> = T::Buf<ptr::Const>;
pub type BufMut<T: Instance> = T::Buf<ptr::Mut>;

// fn encode_to_owned<T: Codable>(value: &T) -> BufOwned<T> {
//     let mut buf = T::buf(unsafe { bytes::Owned::new(vec![0; T::len()].into_boxed_slice()) });
//     value.encode(T::buf_owned_as_mut(&mut buf));
//     buf
// }

pub fn encode_to_array<T: Instance>(value: &T) -> [u8; T::LEN] {
    let mut array = [0; T::LEN];
    value.encode(unsafe { T::buf(ptr::Mut::from_slice(&mut array)) });
    array
}

pub fn buf_swap<T: Instance>(a: BufMut<T>, b: BufMut<T>) {
    T::buf_ptr(a).swap(T::buf_ptr(b))
}

pub fn buf_copy_to<T: Instance>(src: BufConst<T>, dst: BufMut<T>) {
    T::buf_ptr(src).copy_to(T::buf_ptr(dst))
}

pub fn decode<T: Decode, P: Ptr>(buf: T::Buf<P>) -> T {
    T::decode(buf_to_const::<T, P>(buf))
}

pub unsafe fn decode_ptr<T: Decode>(ptr: ptr::Const) -> T {
    T::decode(T::buf(ptr))
}

pub unsafe fn decode_slice<T: Decode>(slice: &[u8]) -> T {
    decode_ptr::<T>(ptr::Const::from_slice(slice))
}

pub unsafe fn encode_ptr<T: Instance>(ptr: ptr::Mut, value: &T) {
    T::encode(value, T::buf(ptr))
}

pub unsafe fn encode_slice<T: Instance>(slice: &mut [u8], value: impl Readable<T>) {
    value.write_to(T::buf(ptr::Mut::from_slice(slice)))
}

pub trait Instance: Entry {
    const LEN: usize;
    fn encode(&self, buf: BufMut<Self>);
    
    // Future plans.
    // fn is_valid(buf: BufConst<Self>) -> bool {
    //     unimplemented!()
    // }
    // fn decode_checked(buf: BufConst<Self>) -> Option<Self> {
    //     if Self::is_valid(buf) {
    //         Some(Self::decode(buf))
    //     } else {
    //         None
    //     }
    // }

    // fn encode_to_owned(&self) -> BufOwned<Self> where Self: Sized {
    //     encode_to_owned(self)
    // }
}

pub trait Decode: Instance {
    fn decode(buf: BufConst<Self>) -> Self;
}

pub trait Readable<T: Instance> {
    fn write_to(self, buf: BufMut<T>);
}

impl<T: Instance> Readable<T> for &T {
    fn write_to(self, buf: BufMut<T>) {
        self.encode(buf);
    }
}

pub trait BufPartialEq<T: Instance>: Readable<T> + Sized {
    fn buf_eq(self, rhs: BufConst<T>) -> bool;
}

pub trait BufEq<T: Instance>: BufPartialEq<T> {}

impl<T: Decode + PartialEq> BufPartialEq<T> for &T {
    default fn buf_eq(self, rhs: BufConst<T>) -> bool {
        self == &T::decode(rhs)
    }
}

impl<T: Decode + Eq> BufEq<T> for &T {}

pub trait BufPartialOrd<T: Instance>: BufPartialEq<T> {
    fn buf_partial_cmp(self, rhs: BufConst<T>) -> Option<Ordering>;
    fn buf_lt(self, rhs: BufConst<T>) -> bool {
        matches!(self.buf_partial_cmp(rhs), Some(Ordering::Less))
    }
    fn buf_gt(self, rhs: BufConst<T>) -> bool {
        matches!(self.buf_partial_cmp(rhs), Some(Ordering::Greater))
    }
}

pub trait BufOrd<T: Instance>: BufPartialOrd<T> {
    fn buf_cmp(self, rhs: BufConst<T>) -> Ordering;
}

impl<T: Decode + PartialOrd> BufPartialOrd<T> for &T {
    default fn buf_partial_cmp(self, rhs: BufConst<T>) -> Option<Ordering> {
        self.partial_cmp(&T::decode(rhs))
    }
}

impl<T: Decode + Ord> BufOrd<T> for &T {
    fn buf_cmp(self, rhs: BufConst<T>) -> Ordering {
        self.cmp(&T::decode(rhs))
    }
}
