use std::cmp::Ordering;

pub use bytes::Value as Bytes;
pub use id::Value as Id;

pub mod bytes;
pub mod id;

pub mod impls;

pub type Buf<T: Instance, BV: bytes::Variant> = T::Buf<BV>;
pub type BufConst<T: Instance> = T::Buf<bytes::variant::Const>;
pub type BufMut<T: Instance> = T::Buf<bytes::variant::Mut>;
pub type BufOwned<T: Instance> = T::Buf<bytes::variant::Owned>;

// pub unsafe fn buf_detach<'a, BV: bytes::variant::Ref, T: Instance>(buf: T::Buf<BV>) -> T::Buf<BV::Ref<'a>> {
//     T::buf_detach(buf)
// }

pub fn buf_swap<T: Instance>(a: BufMut<'_, T>, b: BufMut<'_, T>) {
    T::buf_swap(a, b)
}

fn encode_to_owned<T: Codable>(value: &T) -> BufOwned<T> {
    let mut buf = T::buf(unsafe { bytes::Owned::new(vec![0; T::len()].into_boxed_slice()) });
    value.encode(T::buf_owned_as_mut(&mut buf));
    buf
}

pub trait Instance {
    fn len() -> usize;
    // const LEN: usize;
    type Buf<BV: bytes::Variant>: Clone + Copy;
    fn buf<BV: bytes::Variant>(bytes: Bytes<BV>) -> Self::Buf<BV>;
    // fn buf_rb_const(buf: &'a BufConst<Self>) -> BufConst<Self>;
    // fn buf_rb_mut(buf: &'a mut BufMut<Self>) -> BufMut<Self>;
    fn buf_as_const<BV: bytes::variant::AsConst>(buf: &Buf<Self, BV>) -> BufConst<Self>;
    fn buf_as_mut<BV: bytes::variant::AsMut>(buf: &mut Buf<Self, BV>) -> BufMut<Self>;
    // unsafe fn buf_detach<'b, BV: bytes::variant::Ref>(buf: Self::Buf<BV>) -> Self::Buf<BV::Ref<'b>>;
    fn buf_copy_to(src: BufConst<Self>, dst: BufMut<Self>);
    // unsafe fn buf_copy_nonoverlapping_to() {}
    fn buf_swap(a: BufMut<Self>, b: BufMut<Self>);
}

pub trait BufInstance {
}

impl<T: Instance, BV: bytes::Variant> BufInstance for T::Buf<BV> {

}

pub trait BufInstanceAsConst: BufInstance {
    fn as_const(&self) -> BufConst<Self>;
}

pub trait BufInstanceAsMut: BufInstance {
    fn as_mut(&mut self) -> BufMut<Self>;
}

impl<T: Instance, BV: bytes::Variant> BufInstance for T::Buf<BV> {
    
}

impl<T: Instance, BV: bytes::variant::AsConst> BufInstanceAsConst for T::Buf<BV> {
    fn as_const(&self) -> BufConst<Self> {
        T::buf_as_const(self)
    }
}

impl<T: Instance, BV: bytes::variant::AsConst> BufInstanceAsMut for T::Buf<BV> {
    fn as_mut(&mut self) -> BufMut<Self> {
        T::buf_as_mut(self)
    }
}


// pub trait BufEq: Instance {
//     fn buf_eq<'a>(a: BufConst<'a, Self>, b: BufConst<'a, Self>) -> bool;
// }

// pub trait BufOrd: Instance {
//     fn buf_cmp<'a>(a: BufConst<'a, Self>, b: BufConst<'a, Self>) -> Ordering;
// }

// pub trait BufCopyTo: Instance {
    
// }

// pub trait BufSwap: Instance {
    
// }

pub trait Codable: Instance {
    fn encode(&self, buf: BufMut<Self>);
    fn decode(buf: BufConst<Self>) -> Self;

    fn encode_to_owned(&self) -> BufOwned<Self> where Self: Sized {
        encode_to_owned(self)
    }
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
