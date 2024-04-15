use std::marker::PhantomData;

use entry as instance;

instance! {
    type For = u8;
    pub type Buf = U8Buf;
    const LEN: usize = 1;
}

impl super::Codable for u8 {
    fn encode(&self, buf: super::BufMut<'_, Self>) {
        *buf.0.as_array_mut() = self.to_be_bytes();
    }
    fn decode(buf: super::BufConst<'_, Self>) -> Self {
        Self::from_be_bytes(*buf.0.as_array())
    }
}

instance! {
    type For = u32;
    pub type Buf = U32Buf;
    const LEN: usize = 4;
}

impl super::Codable for u32 {
    fn encode(&self, buf: super::BufMut<'_, Self>) {
        *buf.0.as_array_mut() = self.to_be_bytes();
    }
    fn decode(buf: super::BufConst<'_, Self>) -> Self {
        Self::from_be_bytes(*buf.0.as_array())
    }
}

instance! {
    type For = u64;
    pub type Buf = U64Buf;
    const LEN: usize = 8;
}

impl super::Codable for u64 {
    fn encode(&self, buf: super::BufMut<'_, Self>) {
        *buf.0.as_array_mut() = self.to_be_bytes();
    }
    fn decode(buf: super::BufConst<'_, Self>) -> Self {
        Self::from_be_bytes(*buf.0.as_array())
    }
}


instance! {
    type For<T> = PhantomData<T>;
    pub type Buf = PhantomDataBuf;
    const LEN: usize = 0;
}

impl<T> super::Codable for PhantomData<T> {
    fn encode(&self, buf: super::BufMut<'_, Self>) {
    }
    fn decode(buf: super::BufConst<'_, Self>) -> Self {
        PhantomData
    }
}
