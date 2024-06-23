use {entry as instance, entry_buf as buf};
use std::marker::PhantomData;
use super::Codable;

instance! {
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

instance! {
    buf! { pub struct U32Buf<P>(u32, P); }
    impl I for u32 {
        const LEN: usize = 4;
        type Buf<P> = U32Buf<P>;
    }
}

impl Codable for u32 {
    fn encode(&self, mut buf: super::BufMut<Self>) {
        buf.0.copy_from_slice(&self.to_be_bytes());
    }

    fn decode(buf: super::BufConst<Self>) -> Self {
        Self::from_be_bytes(*unsafe { buf.0.array() })
    }
}

instance! {
    buf! { pub struct U64Buf<P>(u64, P); }
    impl I for u64 {
        const LEN: usize = 8;
        type Buf<P> = U64Buf<P>;
    }
}

impl Codable for u64 {
    fn encode(&self, mut buf: super::BufMut<Self>) {
        buf.0.copy_from_slice(&self.to_be_bytes());
    }

    fn decode(buf: super::BufConst<Self>) -> Self {
        Self::from_be_bytes(*unsafe { buf.0.array() })
    }
}

// instance! {
//     #[derive(Clone, Debug)]
//     struct OptionLike<T: super::Instance> {
//         #[lens(pub field_idk)]
//         idk: T,
//         #[lens(pub field_id)]
//         id: u32,
//     }
    
//     buf! { pub struct OptionLikeBuf<BV, T: super::Instance>(OptionLike<T>, BV); }

//     impl<T: super::Instance> I for OptionLike<T> {
//         type Buf<BV> = OptionLikeBuf<BV, T>;
//     }

//     impl<T: super::Instance + super::Codable> Codable for OptionLike<T> {}
// }

// instance! {
//     struct Wow<T: super::Instance>(OptionLike<T>, u32);
//     buf! { pub struct WowBuf<BV, T: super::Instance>(Wow<T>, BV); }
//     impl<T: super::Instance> I for Wow<T> {
//         type Buf<BV> = WowBuf<BV, T>;
//     }
//     impl<T: super::Instance + super::Codable> Codable for Wow<T> {}
// }
