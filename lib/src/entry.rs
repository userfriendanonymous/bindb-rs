use std::cmp::Ordering;

pub use id::Value as Id;
pub use ptr::Instance as Ptr;

pub mod ptr;
pub mod id;

pub mod impls;

pub type Buf<T: Instance, P> = T::Buf<P>;
pub type BufConst<T: Instance> = T::Buf<ptr::Const>;
pub type BufMut<T: Instance> = T::Buf<ptr::Mut>;

// fn encode_to_owned<T: Codable>(value: &T) -> BufOwned<T> {
//     let mut buf = T::buf(unsafe { bytes::Owned::new(vec![0; T::len()].into_boxed_slice()) });
//     value.encode(T::buf_owned_as_mut(&mut buf));
//     buf
// }

pub fn buf_swap<T: Instance>(a: BufMut<T>, b: BufMut<T>) {
    T::buf_ptr(a).swap(T::buf_ptr(b))
}

pub fn buf_copy_to<T: Instance>(src: BufConst<T>, dst: BufMut<T>) {
    T::buf_ptr(src).copy_to(T::buf_ptr(dst))
}

pub fn buf_to_const<T: Instance, P: Ptr>(buf: Buf<T, P>) -> BufConst<T> {
    T::buf(T::buf_ptr(buf).to_const())
}

pub trait Instance {
    type Buf<P: Ptr>: Clone + Copy;
    const LEN: usize;
    // fn len() -> usize;
    fn buf<P: Ptr>(ptr: P) -> Self::Buf<P>;
    fn buf_ptr<P: Ptr>(buf: Self::Buf<P>) -> P;
}

pub trait Codable: Instance {
    fn encode(&self, buf: BufMut<Self>);
    fn decode(buf: BufConst<Self>) -> Self;
    
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

pub trait Readable<T: Instance> {
    fn write_to(self, buf: BufMut<T>);
}

impl<T: Codable> Readable<T> for &T {
    fn write_to(self, buf: BufMut<T>) {
        self.encode(buf);
    }
}
