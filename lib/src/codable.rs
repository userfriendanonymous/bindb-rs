
pub mod basic;
// pub mod bytes;
use super::buf;

pub trait Instance where Self: Sized {
    const SIZE: usize;

    fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Self>);
    fn decode(bytes: &buf::bytes::Ref<'_, Self>) -> Self;

    unsafe fn decode_unchecked(bytes: &buf::bytes::Ref<'_, Self>) -> Self {
        Self::decode(bytes)
    }
}
