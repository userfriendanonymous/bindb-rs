use std::cmp::Ordering;

pub use crate::bytes_ptr as ptr;
pub use ptr::Instance as Ptr;

pub type Buf<T: Instance, P> = T::Buf<P>;
pub type BufConst<T: Instance> = T::Buf<ptr::Const>;
pub type BufMut<T: Instance> = T::Buf<ptr::Mut>;

pub fn buf_to_const<T: Instance, P: Ptr>(buf: Buf<T, P>) -> BufConst<T> {
    unsafe { T::buf(T::buf_ptr(buf).to_const()) }
}

pub unsafe fn buf_from_slice<T: Instance>(slice: &[u8]) -> BufConst<T> {
    T::buf(ptr::Const::from_slice(slice))
}

pub unsafe fn buf_mut_from_slice<T: Instance>(slice: &mut [u8]) -> BufMut<T> {
    T::buf(ptr::Mut::from_slice(slice))
}

pub trait Instance {
    type Buf<P: Ptr>: Clone + Copy;
    // Caller must ensure ptr is of correct length.
    unsafe fn buf<P: Ptr>(ptr: P) -> Self::Buf<P>;
    fn buf_ptr<P: Ptr>(buf: Self::Buf<P>) -> P;
}

// pub trait BufPartialOrd: Instance {
//     fn buf_partial_cmp<P: Ptr>(a: Buf<Self, P>, b: Buf<Self, P>) -> Option<Ordering>;
// }

// pub trait BufOrd: BufPartialOrd {
//     fn buf_cmp<P: Ptr>(a: Buf<Self, P>, b: Buf<Self, P>) -> Option<Ordering>;
// }
