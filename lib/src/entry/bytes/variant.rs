use std::marker::PhantomData;

pub type Inner<T: Instance> = T::Inner;

pub trait Instance {
    type Inner;
    unsafe fn index_range(inner: Self::Inner, at: usize, len: usize) -> Self::Inner;
}

pub trait Ref: Instance {
    type Ref<'a>: Instance;

    unsafe fn detach<'a>(value: Self::Inner) -> Inner<Self::Ref<'a>>;
}

// pub trait AsConst: Instance {
//     fn as_const<'a>(value: &'a Self::Inner) -> Inner<Const<'a>>;
// }

// pub trait AsMut: AsConst {
//     fn as_mut<'a, 'b>(value: &'b mut Self::Inner) -> Inner<Mut<'a>>;
// }

pub struct Const<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Const<'a> {
    type Inner = &'a [u8];
    unsafe fn index_range(inner: Self::Inner, at: usize, len: usize) -> Self::Inner {
        inner.get_unchecked(at .. at + len)
    }
}

impl<'c> Ref for Const<'c> {
    type Ref<'b> = Const<'b>;

    unsafe fn detach<'a>(value: Self::Inner) -> Inner<Self::Ref<'a>> {
        &*(value as *const _)
    }
}

// impl<'c> AsConst for Const<'c> {
//     fn as_const<'a, 'b>(value: &'b Self::Inner) -> Inner<Const<'a>> {
//         value
//     }
// }

pub struct Mut<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Mut<'a> {
    type Inner = &'a mut [u8];
    unsafe fn index_range(inner: Self::Inner, at: usize, len: usize) -> Self::Inner {
        inner.get_unchecked_mut(at .. at + len)
    }
}

impl<'b> Ref for Mut<'b> {
    type Ref<'a> = Mut<'a>;
    unsafe fn detach<'a>(value: Self::Inner) -> Inner<Self::Ref<'a>> {
        &mut *(value as *mut _)
    }
}

// impl<'c> AsConst for Mut<'c> {
//     fn as_const<'a, 'b>(value: &'b Self::Inner) -> Inner<Const<'a>> {
//         value
//     }
// }

// impl<'c> AsMut for Mut<'c> {
//     fn as_mut<'a, 'b>(value: &'b mut Self::Inner) -> Inner<Mut<'a>> {
//         value
//     }
// }

pub struct Owned;

impl Instance for Owned {
    type Inner = Box<[u8]>;

    unsafe fn index_range(inner: Self::Inner, at: usize, len: usize) -> Self::Inner {
        inner.get_unchecked(at .. at + len).into()
    }
}

// impl AsConst for Owned {
//     fn as_const<'a, 'b>(value: &'b Self::Inner) -> Inner<Const<'a>> {
//         value
//     }
// }

// impl AsMut for Owned {
//     fn as_mut<'a, 'b>(value: &'b mut Self::Inner) -> Inner<Mut<'a>> {
//         value
//     }
// }
