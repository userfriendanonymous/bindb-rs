use std::{array, marker::PhantomData};
use crate::{bytes_ptr, fixed::{self}, BytesPtr, Entry, Fixed};
pub use arb_num::Value as ArbNum;

pub mod primitive;
pub mod dynamic;
pub mod arb_num;

fixed! {
    buf! { pub struct UnitBuf<P>((), P); }
    impl I for () {
        type Buf<P> = UnitBuf<P>;
    }
}

impl Fixed for () {
    const LEN: usize = 0;
    fn encode(&self, _buf: fixed::BufMut<Self>) {}
}
impl fixed::Decode for () {
    fn decode(_buf: fixed::BufConst<Self>) -> Self {}
}

pub struct OptionBuf<P: BytesPtr, T>(P, PhantomData<T>);

impl<P: BytesPtr, T> Clone for OptionBuf<P, T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<P: BytesPtr, T> Copy for OptionBuf<P, T> {}

impl<T> crate::Entry for Option<T> {
    type Buf<P: fixed::Ptr> = OptionBuf<P, T>;
    unsafe fn buf<P: fixed::Ptr>(ptr: P) -> Self::Buf<P> {
        OptionBuf(ptr, PhantomData)
    }
    fn buf_ptr<P: fixed::Ptr>(buf: Self::Buf<P>) -> P {
        buf.0
    }
}

impl<P: BytesPtr, T: Fixed> fixed::Readable<Option<T>> for OptionBuf<P, T> {
    fn write_to(self, buf: fixed::BufMut<Option<T>>) {
        fixed::buf_copy_to::<Option<T>>(unsafe { Option::<T>::buf(buf.0.to_const()) }, buf);
    }
}

impl<T: Fixed> Fixed for Option<T> {
    const LEN: usize = T::LEN + 1;
    fn encode(&self, buf: fixed::BufMut<Self>) {
        match self {
            Some(data) => {
                buf.0.slice()[0] = 1;
                unsafe { fixed::encode_ptr::<T>(buf.0.range_from(1), data) };
            },
            None => {
                buf.0.slice()[0] = 0;
            }
        }
    }
}

impl<T: fixed::Decode> fixed::Decode for Option<T> {
    fn decode(buf: fixed::BufConst<Self>) -> Self {
        match buf.0.slice()[0] {
            1 => {
                Some(unsafe { fixed::decode_ptr::<T>(buf.0.range_from(1)) })
            },
            _ => None,
        }
    }
}

fixed! {
    buf! { pub struct PhantomDataBuf<P, T>(PhantomData<T>, P); }
    impl<T> I for PhantomData<T> {
        type Buf<P> = PhantomDataBuf<P, T>;
    }
}

impl<T> Fixed for PhantomData<T> {
    const LEN: usize = 0;
    fn encode(&self, _buf: fixed::BufMut<Self>) { }
}
impl<T> fixed::Decode for PhantomData<T> {
    fn decode(_buf: fixed::BufConst<Self>) -> Self {
        PhantomData
    }
}

fixed! {
    buf! { pub struct ArrayBuf<P, T: Fixed, const N: usize>([T; N], P); }
    impl<T: Fixed, const N: usize> I for [T; N] {
        type Buf<P> = ArrayBuf<P, T, N>;
    }
}

impl<T: Fixed, const N: usize> Fixed for [T; N] {
    const LEN: usize = N * T::LEN;
    default fn encode(&self, buf: fixed::BufMut<Self>) {
        for idx in 0 .. N {
            unsafe { fixed::encode_ptr(buf.0.range_at(idx * T::LEN, T::LEN), self.get_unchecked(idx)); }
        }
    }
}
impl<T: fixed::Decode, const N: usize> fixed::Decode for [T; N] {
    default fn decode(buf: fixed::BufConst<Self>) -> Self {
        array::from_fn(|idx| {
            unsafe { fixed::decode_ptr(buf.0.range_at(idx * T::LEN, T::LEN)) }
        })
    }
}

impl<const N: usize> Fixed for [u8; N] {
    fn encode(&self, buf: fixed::BufMut<Self>) {
        buf.0.copy_from_slice(self);
    }
}
impl<const N: usize> fixed::Decode for [u8; N] {
    fn decode(buf: fixed::BufConst<Self>) -> Self {
        unsafe { *buf.0.array() }
    }
}
