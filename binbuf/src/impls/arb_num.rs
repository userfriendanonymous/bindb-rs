
// pub struct Value<T: Base, const B: usize>(T);

// pub trait Base {
//     fn to_le_bytes<const B: usize>(&self) -> [u8; B];
//     fn from_le_bytes<const B: usize>(bytes: [u8; B]) -> Self;
// }

// // Value<u32, 4> Value<u64, 3>

use crate::utils::slice_to_array;

pub trait Base {
    const LEN: usize;
    type Bytes;
    fn to_le_bytes(&self) -> Self::Bytes;
    fn from_le_bytes(bytes: Self::Bytes) -> Self;
    fn bytes_to_slice(bytes: &Self::Bytes) -> &[u8];
    unsafe fn slice_to_bytes(slice: &[u8]) -> Self::Bytes;
}

// pub trait BaseU64: Base {
//     fn to_u64(self) -> u64;
//     fn from_u64(val: u64) -> Self;
// }

impl Base for u64 {
    const LEN: usize = 8;
    type Bytes = [u8; 8];
    fn to_le_bytes(&self) -> Self::Bytes {
        u64::to_le_bytes(*self)
    }
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        u64::from_le_bytes(bytes)
    }
    fn bytes_to_slice(bytes: &Self::Bytes) -> &[u8] {
        bytes
    }
    unsafe fn slice_to_bytes(slice: &[u8]) -> Self::Bytes {
        *slice_to_array(slice)
    }
}

// impl BaseU64 for u64 {
//     fn to_u64(self) -> u64 {
        
//     }
// }

fixed! {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Value<const LEN: usize, T>(T);
    buf! { pub struct Buf<P, const LEN: usize, T: Base>(Value<LEN, T>, P); }

    impl<const LEN: usize, T: Base> I for Value<LEN, T> {
        type Buf<P> = Buf<P, LEN, T>;
    }
}

impl<const LEN: usize, T> Value<LEN, T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<const LEN: usize, T: Base> crate::Fixed for Value<LEN, T> {
    const LEN: usize = LEN;
    fn encode(&self, buf: crate::fixed::BufMut<Self>) {
        buf.0.copy_from_slice(&T::bytes_to_slice(&self.0.to_le_bytes())[0 .. LEN]);
    }
}

impl<const LEN: usize, T: Base> crate::fixed::Decode for Value<LEN, T>
where [(); T::LEN]: {
    fn decode(buf: crate::fixed::BufConst<Self>) -> Self {
        let mut arr = [0; T::LEN];
        arr[0 .. LEN].copy_from_slice(buf.0.slice());
        Self(T::from_le_bytes(unsafe { T::slice_to_bytes(&arr) }))
    }
}