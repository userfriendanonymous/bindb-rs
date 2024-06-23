use std::cmp::Ordering;

pub use bytes::Value as Bytes;
pub use id::Value as Id;
pub use ptr::Instance as Ptr;

pub mod ptr;
pub mod id;

pub mod impls;

pub type Buf<T: Instance, P: Ptr> = T::Buf<P>;
pub type BufConst<T: Instance> = T::Buf<ptr::Const>;
pub type BufMut<T: Instance> = T::Buf<ptr::Mut>;

// fn encode_to_owned<T: Codable>(value: &T) -> BufOwned<T> {
//     let mut buf = T::buf(unsafe { bytes::Owned::new(vec![0; T::len()].into_boxed_slice()) });
//     value.encode(T::buf_owned_as_mut(&mut buf));
//     buf
// }

pub trait Instance {
    type Buf<P: Ptr>: Clone + Copy;
    const LEN: usize;
    // fn len() -> usize;
    fn buf<P: Ptr>(ptr: P) -> Self::Buf<P>;
    fn buf_ptr<P: Ptr>(buf: Self::Buf<P>) -> P;
}

pub trait BufInstance: Clone + Copy {
    type T: Instance;
    fn to_const(self) -> BufConst<Self::T>;
}

pub trait BufInstanceMut: BufInstance {
    fn swap(self, other: Self);
    fn copy_from(self, src: BufConst<Self::T>);
}

impl<T: Instance, P: Ptr> BufInstance for T::Buf<P> {
    type T = T;
    fn to_const(self) -> BufConst<T> {
        T::buf(T::buf_ptr(self).to_const())
    }
}

impl<T: Instance> BufInstanceMut for BufMut<T> {
    fn swap(self, other: Self) {
        T::buf_ptr(self).swap(T::buf_ptr(other))
    }

    fn copy_from(self, src: BufConst<Self::T>) {
        T::buf_ptr(self).copy_from(T::buf_ptr(src))
    }
}

pub trait Codable: Instance {
    fn encode(&self, buf: BufMut<Self>);
    fn decode(buf: BufConst<Self>) -> Self;

    // fn encode_to_owned(&self) -> BufOwned<Self> where Self: Sized {
    //     encode_to_owned(self)
    // }
}

pub trait Readable<T: Instance> {
    // type BV: bytes::variant::AsConst;
    fn write_to(self, buf: BufMut<T>);
    // fn into_buf(self) -> T::Buf<Self::BV>;
}

impl<T: Codable> Readable<T> for &T {
    // type BV = bytes::variant::Owned;
    fn write_to(self, buf: BufMut<T>) {
        self.encode(buf)
    }

    // fn into_buf(self) -> T::Buf<Self::BV> {
    //     self.encode_to_owned()
    // }
}
