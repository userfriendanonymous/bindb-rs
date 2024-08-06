use crate::{Fixed, entry::{Buf, BufConst, BufMut}, fixed::{self}};

macro_rules! impl_instance_num {
    ($ty: ty, $buf: ident, $len: literal) => {
        fixed! {
            buf! { pub struct $buf<P>($ty, P); }
            impl I for $ty {
                type Buf<P> = $buf<P>;
            }
        }

        impl Fixed for $ty {
            const LEN: usize = $len;
            fn encode(&self, buf: BufMut<Self>) {
                buf.0.copy_from_slice(&self.to_le_bytes());
            }
        }
        impl fixed::Decode for $ty {
            fn decode(buf: BufConst<Self>) -> Self {
                Self::from_le_bytes(*unsafe { buf.0.array() })
            }
        }
    };
}

impl_instance_num!(u8, U8Buf, 1);
impl_instance_num!(u16, U16Buf, 2);
impl_instance_num!(u32, U32Buf, 4);
impl_instance_num!(u64, U64Buf, 8);
impl_instance_num!(u128, U128Buf, 16);

impl_instance_num!(i8, I8Buf, 1);
impl_instance_num!(i16, I16Buf, 2);
impl_instance_num!(i32, I32Buf, 4);
impl_instance_num!(i64, I64Buf, 8);
impl_instance_num!(i128, I128Buf, 16);

impl_instance_num!(f32, F32Buf, 4);
impl_instance_num!(f64, F64Buf, 8);

fixed! {
    buf! { pub struct BoolBuf<P>(bool, P); }
    impl I for bool {
        type Buf<P> = BoolBuf<P>;
    }
}

impl Fixed for bool {
    const LEN: usize = 1;
    fn encode(&self, mut buf: BufMut<Self>) {
        unsafe { *buf.0.slice().get_unchecked_mut(0) = *self as u8 }
    }
}
impl fixed::Decode for bool {
    fn decode(buf: BufConst<Self>) -> Self {
        unsafe { if *buf.0.slice().get_unchecked(0) == 0 { false } else { true } }
    }
}

fixed! {
    buf! { pub struct CharBuf<P>(char, P); }
    impl I for char {
        type Buf<P> = CharBuf<P>;
    }
}

impl Fixed for char {
    const LEN: usize = 4;
    fn encode(&self, buf: BufMut<Self>) {
        self.encode_utf8(buf.0.into());
    }
}
impl fixed::Decode for char {
    fn decode(buf: BufConst<Self>) -> Self {
        match Self::from_u32(u32::from_le_bytes(*unsafe { buf.0.array() })) {
            Some(v) => v,
            None => char::REPLACEMENT_CHARACTER
        }
    }
}