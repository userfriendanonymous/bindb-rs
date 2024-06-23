use std::marker::PhantomData;

pub type Data<T: Instance> = T::Data;

pub trait Instance {
    type Data;
    unsafe fn index_range(data: Self::Data, at: usize, len: usize) -> Self::Data;
    fn to_const(data: Self::Data) -> Data<Const>;
}

pub struct Const;

impl Instance for Const {
    type Data = *const [u8];
    unsafe fn index_range(inner: Self::Data, at: usize, len: usize) -> Self::Data {
        inner.get_unchecked(at .. at + len)
    }
    fn to_const(data: Self::Data) -> Data<Const> {
        data
    }
}

pub struct Mut;

impl Instance for Mut {
    type Data = *mut [u8];
    unsafe fn index_range(inner: Self::Data, at: usize, len: usize) -> Self::Data {
        inner.get_unchecked_mut(at .. at + len)
    }
    fn to_const(data: Self::Data) -> Data<Const> {
        data
    }
}

// pub struct Owned;

// impl Instance for Owned {
//     type Data = Box<[u8]>;

//     unsafe fn index_range(inner: Self::Data, at: usize, len: usize) -> Self::Data {
//         inner.get_unchecked(at .. at + len).into()
//     }
// }

// impl AsConst for Owned {
//     fn as_const(data: &Self::Data) -> Data<Const> {
//         &*data as *const _
//     }
// }

// impl AsMut for Owned {
//     fn as_mut(data: &mut Self::Data) -> Data<Mut> {
//         &mut *data as *mut _
//     }
// }