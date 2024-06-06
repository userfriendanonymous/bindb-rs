use {entry as instance, entry_buf as buf};
use std::marker::PhantomData;
use super::Codable;

instance! {
    buf! { pub struct PhantomDataBuf<BV, T>(PhantomData<T>, BV); }
    impl<T> I for PhantomData<T> {
        type Buf<BV> = PhantomDataBuf<BV, T>;
        fn len() -> usize {
            0
        }
    }
}

impl<T> Codable for PhantomData<T> {
    fn encode(&self, _buf: super::BufMut<'_, Self>) { }
    fn decode(_buf: super::BufConst<'_, Self>) -> Self {
        PhantomData
    }
}

instance! {
    buf! { pub struct U32Buf<BV: super::bytes::Variant>(u32, BV); }
    impl I for u32 {
        type Buf<BV> = U32Buf<BV>;
        fn len() -> usize {
            4
        }
    }
}

impl Codable for u32 {
    fn encode(&self, buf: super::BufMut<'_, Self>) {
        buf.0.copy_from_slice(&self.to_be_bytes());
    }

    fn decode(buf: super::BufConst<'_, Self>) -> Self {
        Self::from_be_bytes(*unsafe { buf.0.array() })
    }
}

instance! {
    buf! { pub struct U64Buf<BV: super::bytes::Variant>(u64, BV); }
    impl I for u64 {
        type Buf<BV> = U64Buf<BV>;
        fn len() -> usize {
            8
        }
    }
}

impl Codable for u64 {
    fn encode(&self, buf: super::BufMut<'_, Self>) {
        buf.0.copy_from_slice(&self.to_be_bytes());
    }

    fn decode(buf: super::BufConst<'_, Self>) -> Self {
        Self::from_be_bytes(*unsafe { buf.0.array() })
    }
}

instance! {
    struct OptionLike<T: super::Instance> {
        #[lens(pub field_idk)]
        idk: T,
        #[lens(pub field_id)]
        id: u32,
    }
    buf! { pub struct OptionLikeBuf<BV, T: super::Instance>(OptionLike<T>, BV); }

    impl<T: super::Instance> I for OptionLike<T> {
        type Buf<BV> = OptionLikeBuf<BV, T>;
    }

    impl<T: super::Instance + super::Codable> Codable for OptionLike<T> {}
}

instance! {
    struct Wow<T: super::Instance>(OptionLike<T>, u32);
    buf! { pub struct WowBuf<BV, T: super::Instance>(Wow<T>, BV); }
    impl<T: super::Instance> I for Wow<T> {
        type Buf<BV> = WowBuf<BV, T>;
    }
    impl<T: super::Instance + super::Codable> Codable for Wow<T> {}
}


pub fn idk() {
    type Idk = <OptionLike::<u32> as crate::entry::Instance>::Buf<crate::entry::bytes::variant::Const<'static>>;
    OptionLike::<u32>::field_idk::<crate::entry::bytes::variant::Const<'static>>(todo!());
}