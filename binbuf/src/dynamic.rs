#![allow(type_alias_bounds)]

pub use crate::{Entry, bytes_ptr as ptr};
pub use ptr::Instance as Ptr;

pub mod lens;

pub type Buf<T: Instance, P> = T::Buf<P>;
pub type BufConst<T: Instance> = T::Buf<ptr::Const>;
pub type BufMut<T: Instance> = T::Buf<ptr::Mut>;

pub fn buf_len<T: Instance>(buf: BufConst<T>) -> usize {
    T::buf_len(buf)
}

pub unsafe fn ptr_len<T: Instance>(ptr: ptr::Const) -> usize {
    T::buf_len(T::buf(ptr))
}

pub fn buf_swap<T: Instance>(a: BufMut<T>, b: BufMut<T>) {
    T::buf_ptr(a).swap(T::buf_ptr(b))
}

pub fn buf_copy_to<T: Instance>(src: BufConst<T>, dst: BufMut<T>) -> usize {
    unsafe {
        let len = T::buf_len(src);
        T::buf_ptr(src).range_at(0, len).copy_to(T::buf_ptr(dst).range_at(0, len));
        len
    }
}

pub fn buf_to_const<T: Instance, P: Ptr>(buf: Buf<T, P>) -> BufConst<T> {
    unsafe { T::buf(T::buf_ptr(buf).to_const()) }
}

pub unsafe fn decode_ptr<T: Decode>(ptr: ptr::Const) -> (T, usize) {
    T::decode(T::buf(ptr))
}

pub fn decode<T: Decode>(buf: BufConst<T>) -> (T, usize) {
    T::decode(buf)
}

pub unsafe fn decode_slice<T: Decode>(slice: &[u8]) -> (T, usize) {
    decode_ptr::<T>(ptr::Const::from_slice(slice))
}

pub unsafe fn encode_ptr<T: Instance>(ptr: ptr::Mut, value: &T) -> usize {
    T::encode(value, T::buf(ptr))
}

pub trait Instance: Entry + Sized {
    fn len(&self) -> usize;
    fn buf_len(buf: BufConst<Self>) -> usize;
    fn encode(&self, buf: BufMut<Self>) -> usize;
}

pub trait Decode: Instance {
    fn decode(buf: BufConst<Self>) -> (Self, usize);
}

impl<T: crate::Fixed> Instance for T {
    default fn len(&self) -> usize {
        T::LEN
    }
    default fn buf_len(buf: BufConst<Self>) -> usize {
        T::LEN
    }
    default fn encode(&self, buf: BufMut<Self>) -> usize {
        <T as crate::Fixed>::encode(&self, unsafe { T::buf(T::buf_ptr(buf).range_at(0, T::LEN)) });
        T::LEN
    }
}

impl<T: crate::fixed::Decode> Decode for T {
    default fn decode(buf: BufConst<Self>) -> (Self, usize) {
        let buf = unsafe { T::buf(T::buf_ptr(buf).range_at(0, T::LEN)) };
        (<T as crate::fixed::Decode>::decode(buf), T::LEN)
    }
}

// pub struct BufWithLen<T: Instance, P: Ptr>(T::Buf<P>, usize);

pub trait Readable<T: Instance> {
    fn len(&self) -> usize;
    fn write_to(self, buf: BufMut<T>) -> usize;
}

impl<T: Instance> Readable<T> for &T {
    fn len(&self) -> usize {
        T::len(&self)
    }

    fn write_to(self, buf: BufMut<T>) -> usize {
        self.encode(buf)
    }
}
