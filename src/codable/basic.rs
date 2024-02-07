//! Implementations of [`Instance`] for basic types.

use super::Instance;

impl Instance for u32 {
    type DecodeError = ();

    fn size() -> usize {
        4
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

impl Instance for u64 {
    type DecodeError = ();

    fn size() -> usize {
        8
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }
}

impl Instance for i32 {
    type DecodeError = ();

    fn size() -> usize {
        4
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.to_le_bytes())
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

impl Instance for i64 {
    type DecodeError = ();

    fn size() -> usize {
        8
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        Ok(i64::from_le_bytes(bytes.try_into().unwrap()))
    }
}

impl Instance for f32 {
    type DecodeError = ();

    fn size() -> usize {
        4
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        Ok(f32::from_le_bytes(bytes.try_into().unwrap()))
    }
}

impl Instance for f64 {
    type DecodeError = ();

    fn size() -> usize {
        8
    }

    fn encode(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        Ok(f64::from_le_bytes(bytes.try_into().unwrap()))
    }
}

#[derive(Clone, Debug)]
pub enum OptionDecodeError<ChildDecodeError> {
    InvalidOption,
    Child(ChildDecodeError),
}

impl<T: Instance> Instance for Option<T> {
    type DecodeError = OptionDecodeError<T::DecodeError>;

    fn size() -> usize {
        T::size() + 1
    }

    fn encode(&self, bytes: &mut [u8]) {
        match self {
            Some(v) => {
                bytes[0] = 1;
                v.encode(&mut bytes[1..]);
            }
            None => bytes[0] = 0
        };
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        match bytes[0] {
            0 => Ok(None),
            1 => T::decode(&bytes[1..]).map_err(OptionDecodeError::Child).map(Some),
            _ => Err(OptionDecodeError::InvalidOption)
        }
    }
}