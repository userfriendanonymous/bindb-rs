use crate::entry::{Codable, BufConst, Buf, BufMut};

macro_rules! impl_instance_num {
    ($ty: ty, $buf: ident, $len: literal) => {
        entry! {
            buf! { pub struct $buf<P>($ty, P); }
            impl I for $ty {
                const LEN: usize = $len;
                type Buf<P> = $buf<P>;
            }
        }

        impl Codable for $ty {
            fn encode(&self, buf: BufMut<Self>) {
                buf.0.copy_from_slice(&self.to_le_bytes());
            }
        
            fn decode(buf: BufConst<Self>) -> Self {
                Self::from_le_bytes(*unsafe { buf.0.array() })
            }
        }
    };
}

impl_instance_num!(u8, U8Buf, 1);
impl_instance_num!(u32, U32Buf, 4);
impl_instance_num!(u64, U64Buf, 8);
impl_instance_num!(u128, U128Buf, 16);

impl_instance_num!(i8, I8Buf, 1);
impl_instance_num!(i32, I32Buf, 4);
impl_instance_num!(i64, I64Buf, 8);
impl_instance_num!(i128, I128Buf, 16);

impl_instance_num!(f32, F32Buf, 4);
impl_instance_num!(f64, F64Buf, 8);

entry! {
    buf! { pub struct BoolBuf<P>(bool, P); }
    impl I for bool {
        const LEN: usize = 1;
        type Buf<P> = BoolBuf<P>;
    }
}

impl Codable for bool {
    fn encode(&self, mut buf: BufMut<Self>) {
        unsafe { *buf.0.get_unchecked_mut(0) = *self as u8 }
    }

    fn decode(buf: BufConst<Self>) -> Self {
        unsafe { if *buf.0.get_unchecked(0) == 0 { false } else { true } }
    }
}

entry! {
    buf! { pub struct CharBuf<P>(char, P); }
    impl I for char {
        const LEN: usize = 4;
        type Buf<P> = CharBuf<P>;
    }
}

impl Codable for char {
    fn encode(&self, mut buf: BufMut<Self>) {
        unsafe { self.encode_utf8(buf.0.into()); }
    }

    fn decode(buf: BufConst<Self>) -> Self {
        match Self::from_u32(u32::from_le_bytes(*unsafe { buf.0.array() })) {
            Some(v) => v,
            None => char::REPLACEMENT_CHARACTER
        }
    }
}