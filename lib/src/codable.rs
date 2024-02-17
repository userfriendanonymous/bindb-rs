
pub mod basic;
// pub mod bytes;
use super::{buf, lenser};

pub trait Instance where Self: Sized {
    const SIZE: usize;

    fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Self>);
    fn decode(bytes: &buf::bytes::Ref<'_, Self>) -> Self;

    unsafe fn decode_unchecked(bytes: &buf::bytes::Ref<'_, Self>) -> Self {
        Self::decode(bytes)
    }

    type Lenser;
    fn lenser_from_root(root: lenser::Root<Self>) -> Self::Lenser;

    fn lenser() -> Self::Lenser {
        Self::lenser_from_root(lenser::Root::VALUE)
    }
}

pub fn lenser<B: Instance>() -> B::Lenser {
    B::lenser()
}