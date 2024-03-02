pub use bytes::Value as Bytes;
pub use id::Value as Id;

pub mod bytes;
pub mod id;

pub type BufConst<'a, T: Instance> = T::Buf<bytes::variant::Const<'a>>;
pub type BufMut<'a, T: Instance> = T::Buf<bytes::variant::Mut<'a>>;

pub trait Instance {
    const LEN: usize;
    type Buf<BV: bytes::Variant>;
    fn buf<BV: bytes::Variant>(bytes: Bytes<BV, { Self::LEN }>) -> Self::Buf<BV>;
}

pub trait Codable: Instance {
    fn encode<BV: bytes::variant::AsMut>(&self, buf: &mut Self::Buf<BV>);
    fn decode<BV: bytes::variant::AsConst>(buf: &Self::Buf<BV>) -> Self;
}

pub trait Readable<T: Instance> {
    fn write_to<BV: bytes::variant::AsMut>(&self, buf: &mut T::Buf<BV>);
}

impl<T: Codable> Readable<T> for T {
    fn write_to<BV: bytes::variant::AsMut>(&self, buf: &mut <T as Instance>::Buf<BV>) {
        self.encode(buf)
    }
}
