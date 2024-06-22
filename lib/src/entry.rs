use std::cmp::Ordering;

pub use bytes::Value as Bytes;
pub use id::Value as Id;

pub mod bytes;
pub mod id;

pub mod impls;

pub type Buf<T: Instance, BV: bytes::Variant> = T::Buf<BV>;
pub type BufConst<'a, T: Instance> = T::Buf<bytes::variant::Const<'a>>;
pub type BufMut<'a, T: Instance> = T::Buf<bytes::variant::Mut<'a>>;
pub type BufOwned<T: Instance> = T::Buf<bytes::variant::Owned>;

pub unsafe fn buf_detach<'a, BV: bytes::variant::Ref, T: Instance>(buf: T::Buf<BV>) -> T::Buf<BV::Ref<'a>> {
    T::buf_detach(buf)
}

pub fn buf_swap<T: Instance>(a: BufMut<'_, T>, b: BufMut<'_, T>) {
    T::buf_swap(a, b)
}

fn encode_to_owned<T: Codable>(value: &T) -> BufOwned<T> {
    let mut buf = T::buf(bytes::Owned::new(vec![0; T::len()].into_boxed_slice()));
    value.encode(T::buf_owned_as_mut(&mut buf));
    buf
}

pub trait Instance {
    fn len() -> usize;
    // const LEN: usize;
    type Buf<BV: bytes::Variant>;
    fn buf<BV: bytes::Variant>(bytes: Bytes<BV>) -> Self::Buf<BV>;
    fn buf_rb_const<'a>(buf: &'a BufConst<'a, Self>) -> BufConst<'a, Self>;
    fn buf_rb_mut<'a>(buf: &'a mut BufMut<'a, Self>) -> BufMut<'a, Self>;
    fn buf_owned_as_const(buf: &BufOwned<Self>) -> BufConst<'_, Self>;
    fn buf_owned_as_mut(buf: &mut BufOwned<Self>) -> BufMut<'_, Self>;
    fn buf_mut_as_const<'a>(buf: &'a BufMut<'a, Self>) -> BufConst<'a, Self>;
    unsafe fn buf_detach<'b, BV: bytes::variant::Ref>(buf: Self::Buf<BV>) -> Self::Buf<BV::Ref<'b>>;
    fn buf_copy_to(src: BufConst<'_, Self>, dst: BufMut<'_, Self>);
    unsafe fn buf_copy_nonoverlapping_to() {}
    fn buf_swap(a: BufMut<'_, Self>, b: BufMut<'_, Self>);
}

// pub trait BufDetach: Instance {
    
// }

pub trait BufEq: Instance {
    fn buf_eq<'a>(a: BufConst<'a, Self>, b: BufConst<'a, Self>) -> bool;
}

pub trait BufOrd: Instance {
    fn buf_cmp<'a>(a: BufConst<'a, Self>, b: BufConst<'a, Self>) -> Ordering;
}

// pub trait BufCopyTo: Instance {
    
// }

// pub trait BufSwap: Instance {
    
// }

pub trait Codable: Instance {
    fn encode<'a>(&'a self, buf: BufMut<'a, Self>);
    fn decode<'a>(buf: BufConst<'a, Self>) -> Self;

    fn encode_to_owned(&self) -> BufOwned<Self> where Self: Sized {
        encode_to_owned(self)
    }
}

pub trait Readable<T: Instance> {
    // type BV: bytes::variant::AsConst;
    fn write_to(self, buf: BufMut<'_, T>);
    // fn into_buf(self) -> T::Buf<Self::BV>;
}

impl<T: Codable> Readable<T> for &T {
    // type BV = bytes::variant::Owned;
    fn write_to(self, buf: BufMut<'_, T>) {
        self.encode(buf)
    }

    // fn into_buf(self) -> T::Buf<Self::BV> {
    //     self.encode_to_owned()
    // }
}
