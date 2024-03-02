//! Implementations of [`Instance`] for basic types.

use crate::{buf, lenser};
use super::Instance;

impl Instance for u32 {
    const SIZE: usize = 4;

    fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Self>) {
        bytes.copy_from(&(&self.to_le_bytes()).into());
    }

    fn decode(bytes: &buf::bytes::Ref<'_, Self>) -> Self where Self: Sized {
        Self::from_le_bytes(*bytes.as_array())
    }

    type Lenser = lenser::Empty<Self>;
    fn lenser_from_root(root: lenser::Root<Self>) -> Self::Lenser {
        lenser::Empty::new(root)
    }
}

impl Instance for u64 {
    const SIZE: usize = 8;

    fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Self>) {
        bytes.copy_from(&(&self.to_le_bytes()).into());
    }

    fn decode(bytes: &buf::bytes::Ref<'_, Self>) -> Self where Self: Sized {
        Self::from_le_bytes(*bytes.as_array())
    }

    type Lenser = lenser::Empty<Self>;
    fn lenser_from_root(root: lenser::Root<Self>) -> Self::Lenser {
        lenser::Empty::new(root)
    }
}

// impl Instance for u64 {
//     const SIZE: usize = 8;

//     fn encode(&self, bytes: &mut [u8; Self::SIZE]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8; Self::SIZE]) -> Self where Self: Sized {
//         Self::from_le_bytes(bytes.try_into().unwrap())
//     }

//     type Lens = lens::Empty;
//     fn lens_offset(value: Self::Lens) -> usize { match value {} }
// }

// impl Instance for i32 {
//     const SIZE: usize = 4;

//     fn encode(&self, bytes: &mut [u8; Self::SIZE]) {
//         bytes.copy_from_slice(&self.to_le_bytes())
//     }

//     fn decode(bytes: &[u8; Self::SIZE]) -> Self where Self: Sized {
//         Self::from_le_bytes(bytes.try_into().unwrap())
//     }

//     type Lens = lens::Empty;
//     fn lens_offset(value: Self::Lens) -> usize { match value {} }
// }

// impl Instance for i64 {
//     const SIZE: usize = 8;

//     fn encode(&self, bytes: &mut [u8; Self::SIZE]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8; Self::SIZE]) -> Self where Self: Sized {
//         Self::from_le_bytes(*bytes)
//     }

//     type Lens = lens::Empty;
//     fn lens_offset(value: Self::Lens) -> usize { match value {} }
// }

// impl Instance for f32 {
//     const SIZE: usize = 4;

//     fn encode(&self, bytes: &mut [u8; Self::SIZE]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8; Self::SIZE]) -> Self where Self: Sized {
//         f32::from_le_bytes(*bytes)
//     }
// }

// impl Instance for f64 {
//     const SIZE: usize = 8;

//     fn encode(&self, bytes: &mut [u8; Self::SIZE]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8; Self::SIZE]) -> Self where Self: Sized {
//         f64::from_le_bytes(*bytes)
//     }
// }

impl<T: Instance> Instance for Option<T> {
    const SIZE: usize = T::SIZE + 1;

    fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Self>) {
        match self {
            Some(v) => {
                bytes[0] = 1;
                v.encode(&mut bytes.index_to(1));
            }
            None => bytes.fill(0)
        };
    }

    fn decode(bytes: &buf::bytes::Ref<'_, Self>) -> Self where Self: Sized {
        match bytes[0] {
            1 => Some(T::decode(&bytes.index_to(1))),
            _ => None,
        }
    }

    type Lenser = lenser::Empty<Self>;
    fn lenser_from_root(root: lenser::Root<Self>) -> Self::Lenser {
        lenser::Empty::new(root)
    }
}