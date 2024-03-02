pub use bytes::Value as Bytes;
pub mod bytes;

pub trait ImplConst<'a> = Instance<bytes::variant::Const<'a>>;
pub trait IsMut<'a> = Instance<bytes::variant::Mut<'a>>;

pub type Const<'a, T: ConstInst<'a>> = T<bytes::variant::Const<'a>>;
// pub type Mut<'a, T: IsMut<'a>> =;

pub trait Spawner {
    const LEN: usize;
    type Buf<BV: bytes::Variant>: Buf<Spawner = Self>;
    fn from_bytes<BV: bytes::Variant>(bytes: Bytes<BV, { Self::LEN }>) -> Self::Buf<BV>;
}

pub trait Buf {
    type Spawner: Spawner;
}

pub trait Codable {
    fn encode() {}
}

pub trait Instance<BV: bytes::Variant> {
    const LEN: usize;

    fn from_bytes(bytes: Bytes<BV, { Self::LEN }>) -> Self;
}

pub trait Codable {
    type Value;

    fn encode<BV: bytes::variant::AsMut>(&mut self, value: &Self::Value)
    where
        Self: Instance<BV>;

    fn decode<BV: bytes::variant::AsConst>(&self) -> Self::Value
    where
        Self: Instance<BV>;
}

pub trait Readable<T> {
    fn write_to<BV: bytes::variant::AsMut>(&self, buf: &mut T)
    where
        Self: Instance<BV>;
}

impl<T: Codable> Readable<T> for T::Value {
    fn write_to<BV: bytes::variant::AsMut>(&self, buf: &mut T)
    where
        Self: Instance<BV>,
    {
        buf.encode(self)
    }
}

// pub type BufRef<'a, T: Instance> = T::Buf<bytes::Ref<'a, { T::LEN }>>;
// pub type BufMut<'a, T: Instance> = T::Buf<bytes::Mut<'a, { T::LEN }>>;

// // pub trait AsValue<T: Instance> {
// //     fn write(&self, bytes: bytes::Mut<'_, T::LEN>);
// // }

// // impl<T: Instance> AsValue<T> for T {
// //     fn write(&self, bytes: bytes::Mut<'_, T::LEN>) {
// //         self.encode(bytes)
// //     }
// // }

// // impl<'a, T: Instance> AsValue<T> for bytes::Ref<'a, T::LEN> {
// //     fn write(&self, bytes: bytes::Mut<'_, T::LEN>) {
// //         bytes.copy_from(self)
// //     }
// // }

// // impl<'a, T: Instance> AsValue<T> for bytes::Mut<'a, T::LEN> {
// //     fn write(&self, bytes: bytes::Mut<'_, T::LEN>) {
// //         bytes.copy_from(self)
// //     }
// // }

// // impl<'a, T: Instance> AsValue<T> for bytes::Owned<T::LEN> {
// //     fn write(&self, bytes: bytes::Mut<'_, T::LEN>) {
// //         bytes.copy_from(self)
// //     }
// // }

// pub trait Instance {
//     type Buf<B>: Buf<Bytes = B, Of = Self>;
//     fn buf<B>(bytes: B) -> Self::Buf<B>;
// }

// pub trait Readable<T: Instance> {
//     fn write_to<'a>(self, buf: BufMut<'a, T>);
// }

// impl<T: Instance> Readable<T> for &T {
//     fn write_to<'a>(self, mut buf: BufMut<'a, T>) {
//         buf.encode(self)
//     }
// }
