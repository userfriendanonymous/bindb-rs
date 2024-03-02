use std::marker::PhantomData;

pub type ValueOf<T: Instance, const L: usize> = T::Value<L>;

pub trait Instance {
    type Value<const L: usize>;
}

pub trait Ref: Instance {
    type Ref<'a>: Instance;

    unsafe fn detach<'a, const L: usize>(value: Self::Value<L>) -> ValueOf<Self::Ref<'a>, L>;
}

pub trait AsConst: Instance {
    fn as_const<'a, const L: usize>(value: &'a Self::Value<L>) -> ValueOf<Const<'a>, L>;
}

pub trait AsMut: AsConst {
    fn as_mut<'a, const L: usize>(value: &'a mut Self::Value<L>) -> ValueOf<Mut<'a>, L>;
}

pub struct Const<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Const<'a> {
    type Value<const L: usize> = &'a [u8];
}

impl<'c> Ref for Const<'c> {
    type Ref<'b> = Const<'b>;

    unsafe fn detach<'a, const L: usize>(value: Self::Value<L>) -> ValueOf<Self::Ref<'a>, L> {
        &*(value as *const _)
    }
}

impl<'b> AsConst for Const<'b> {
    fn as_const<'a, const L: usize>(value: &'a Self::Value<L>) -> ValueOf<Const<'a>, L> {
        value
    }
}

pub struct Mut<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Mut<'a> {
    type Value<const L: usize> = &'a mut [u8];
}

impl<'b> Ref for Mut<'b> {
    type Ref<'a> = Mut<'a>;
    unsafe fn detach<'a, const L: usize>(value: Self::Value<L>) -> ValueOf<Self::Ref<'a>, L> {
        &mut *(value as *mut _)
    }
}

impl<'b> AsConst for Mut<'b> {
    fn as_const<'a, const L: usize>(value: &'a Self::Value<L>) -> ValueOf<Const<'a>, L> {
        value
    }
}

impl<'b> AsMut for Mut<'b> {
    fn as_mut<'a, const L: usize>(value: &'a mut Self::Value<L>) -> ValueOf<Mut<'a>, L> {
        value
    }
}

pub struct Owned;

impl Instance for Owned {
    type Value<const L: usize> = [u8; L];
}

impl AsConst for Owned {
    fn as_const<'a, const L: usize>(value: &'a Self::Value<L>) -> ValueOf<Const<'a>, L> {
        value
    }
}

impl AsMut for Owned {
    fn as_mut<'a, const L: usize>(value: &'a mut Self::Value<L>) -> ValueOf<Mut<'a>, L> {
        value
    }
}

// use crate::{
//     ownership::{self, Const, Mut},
//     utils::slice_to_array,
// };

// pub trait Ref = ownership::Ref<Base>;
// pub trait AsConst = ownership::AsConst<Base>;
// pub trait AsMut = ownership::AsMut<Base>;

// pub struct Base;

// pub trait Instance: ownership::Instance<Base> {
//     // const LEN: usize;
//     unsafe fn detach<'a>(value: Self::Value) -> Self::Ref<'a>;
//     fn into_owned(value: Self::Value) -> <Owned as ownership::Instance<Base>>::Value;
// }

// impl<const L: usize> ownership::Refable for Base {
//     type New<O: crate::Ownership<Self>> = O::Value;
//     type Const<'a> = &'a [u8];
//     type Mut<'a> = &'a mut [u8];

//     fn mut_as_const<'a>(value: &Self::Mut<'a>) -> Self::Const<'a> {
//         value
//     }

//     fn rb_mut<'a>(value: &mut Self::Mut<'a>) -> Self::Mut<'a> {
//         *value
//     }

//     fn rb_const<'a>(value: &Self::Const<'a>) -> Self::Const<'a> {
//         *value
//     }
// }

// pub type Owned = ownership::Owned<ArrayOwnedIn>;

// pub struct ArrayOwnedIn;

// impl<const L: usize> ownership::Ownable<ArrayOwnedIn> for Base {
//     type Owned = [u8; L];
//     fn owned_as_const<'a>(value: &[u8; L]) -> Self::Const<'a> {
//         value
//     }
//     fn owned_as_mut<'a>(value: &mut [u8; L]) -> Self::Mut<'a> {
//         value
//     }
// }

// impl<'a, const L: usize> Instance for Const<'a> {
//     // const LEN: usize = L;
//     unsafe fn detach<'b>(value: Self::Value) -> Self::Ref<'b> {
//         &*(value as *const _)
//     }

//     fn into_owned(value: Self::Value) -> <Owned as ownership::Instance<Base>>::Value {
//         super::Value(unsafe { slice_to_array(value) }.clone())
//     }
// }

// impl<'a, const L: usize> Instance for Mut<'a> {
//     // const LEN: usize = L;
//     unsafe fn detach<'b>(value: Self::Value) -> Self::Ref<'b> {
//         &mut *(value as *mut _)
//     }

//     fn into_owned(value: Self::Value) -> super::Value<L, Owned> {
//         super::Value(unsafe { slice_to_array(value) }.clone())
//     }
// }

// impl<'a, const L: usize> Instance for Owned {
//     // const LEN: usize = L;
//     unsafe fn detach<'b>(value: Self::Value) -> Self::Ref<'b> {
//         value
//     }

//     fn into_owned(value: Self::Value) -> super::Value<L, Owned> {
//         value
//     }
// }
