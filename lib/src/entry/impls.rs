use std::{array, marker::PhantomData};
pub mod primitive;
use crate::{Entry, entry::{Codable, BufConst, Buf, BufMut, Ptr}};

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
    fn encode(&self, buf: BufMut<Self>) {
        for idx in 0 .. N {
            unsafe { buf.0.index_range(idx * T::LEN, T::LEN).encode(self.get_unchecked(idx)); }
        }
    }

    fn decode(buf: BufConst<Self>) -> Self {
        array::from_fn(|idx| {
            unsafe { buf.0.index_range(idx * T::LEN, T::LEN).decode() }
        })
    }
}