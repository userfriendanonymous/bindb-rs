use std::{array, marker::PhantomData};
use super::{Instance as Entry, Codable, BufConst, Buf, BufMut, Ptr};

pub mod primitive;

entry! {
    buf! { pub struct UnitBuf<P, T>((), P); }
    impl I for () {
        const LEN: usize = 0;
        type Buf<P> = UnitBuf<P>;
    }
}

impl Codable for () {
    fn encode(&self, buf: BufMut<Self>) {}
    fn decode(buf: BufConst<Self>) -> Self {}
}

impl<T> Codable for PhantomData<T> {
    fn encode(&self, _buf: super::BufMut<Self>) { }
    fn decode(_buf: super::BufConst<Self>) -> Self {
        PhantomData
    }
}

entry! {
    buf! { pub struct PhantomDataBuf<P, T>(PhantomData<T>, P); }
    impl<T> I for PhantomData<T> {
        const LEN: usize = 0;
        type Buf<P> = PhantomDataBuf<P, T>;
    }
}

impl<T> Codable for PhantomData<T> {
    fn encode(&self, _buf: super::BufMut<Self>) { }
    fn decode(_buf: super::BufConst<Self>) -> Self {
        PhantomData
    }
}

entry! {
    buf! { pub struct ArrayBuf<P, T: Entry, const N: usize>([T; N], P); }
    impl<T: Entry, const N: usize> I for [T; N] {
        const LEN: usize = N * T::LEN;
        type Buf<P> = ArrayBuf<P, T, N>;
    }
}

impl<T: Codable, const N: usize> Codable for [T; N] {
    default fn encode(&self, buf: BufMut<Self>) {
        for idx in 0 .. N {
            unsafe { buf.0.index_range(idx * T::LEN, T::LEN).encode(self.get_unchecked(idx)); }
        }
    }

    default fn decode(buf: BufConst<Self>) -> Self {
        array::from_fn(|idx| {
            unsafe { buf.0.index_range(idx * T::LEN, T::LEN).decode() }
        })
    }
}

impl<const N: usize> Codable for [u8; N] {
    fn encode(&self, buf: BufMut<Self>) {
        buf.0.copy_from_slice(self);
    }

    fn decode(buf: BufConst<Self>) -> Self {
        unsafe { *buf.0.array() }
    }
}
