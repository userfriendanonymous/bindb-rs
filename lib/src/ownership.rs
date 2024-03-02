use std::marker::PhantomData;

pub trait Refable {
    type New<O: Instance<Self>>;
    type Const<'a>;
    type Mut<'a>;

    fn mut_as_const<'a>(value: &Self::Mut<'a>) -> Self::Const<'a>;

    fn rb_const<'a>(value: &Self::Const<'a>) -> Self::Const<'a>;
    fn rb_mut<'a>(value: &mut Self::Mut<'a>) -> Self::Mut<'a>;
}

pub trait Ownable<In>: Refable {
    type Owned;
    fn owned_as_mut<'a>(value: &mut Self::Owned) -> Self::Mut<'a>;
    fn owned_as_const<'a>(value: &Self::Owned) -> Self::Const<'a>;
}

pub trait Ref<T> {
    type Ref<'a>;
}

pub trait Instance<T>: Ref<T> {
    type Value;
}

pub trait AsConst
where
    Self: Instance,
{
    fn as_const(value: &Self::Value) -> Self::Base::Const<'_>;
}

pub trait AsMut<T: Refable>: AsConst<T>
where
    Self: Instance<T>,
{
    fn as_mut(value: &mut Self::Value) -> T::Mut<'_>;
}

// impl<'a, D: Dif> AsConst<'a> for D::Const<'a> {
//     type D = D;
//     fn as_const(&self) -> <D as Dif>::Const<'a> {
//         D::rb_const(self)
//     }
// }

// impl<'a, D: Dif> AsConst<'a> for D::Mut<'a> {
//     type D = D;
//     fn as_const(&self) -> <D as Dif>::Const<'a> {
//         D::mut_as_const(self)
//     }
// }

// impl<'a, D: Dif> AsConst<'a> for D::Owned {
//     type D = D;
//     fn as_const(&self) -> <D as Dif>::Const<'a> {
//         D::owned_as_ref(self)
//     }
// }

// pub trait AsMut<D: Dif> {
//     fn as_mut<'a>(&'a self) -> D::Mut<'a>;
// }

pub struct Const<'a>(PhantomData<&'a ()>);

impl<'b, T: Refable> Ref<T> for Const<'b> {
    type Ref<'a> = T::Const<'a>;
}

impl<'a, T: Refable> Instance<T> for Const<'a> {
    type Value = T::Const<'a>;
}

impl<'a, T: Refable> AsConst<T> for Const<'a> {
    fn as_const(value: &Self::Value) -> <T as Refable>::Const<'_> {
        T::rb_const(value)
    }
}

pub struct Mut<'a>(PhantomData<&'a ()>);

impl<'b, T: Refable> Ref<T> for Mut<'b> {
    type Ref<'a> = T::Mut<'a>;
}

impl<'a, T: Refable> Instance<T> for Mut<'a> {
    type Value = T::Mut<'a>;
}

impl<'a, T: Refable> AsConst<T> for Mut<'a> {
    fn as_const(value: &Self::Value) -> <T as Refable>::Const<'_> {
        T::mut_as_const(value)
    }
}

impl<'a, T: Refable> AsMut<T> for Mut<'a> {
    fn as_mut(value: &mut Self::Value) -> <T as Refable>::Mut<'_> {
        T::rb_mut(value)
    }
}

pub struct Owned<In>(PhantomData<In>);

impl<In, T: Ownable<In>> Instance<T> for Owned<In> {
    type Value = T::Owned;
}

impl<In, T: Ownable<In>> AsConst<T> for Owned<In> {
    fn as_const(value: &Self::Value) -> <T as Refable>::Const<'_> {
        T::owned_as_const(value)
    }
}

impl<In, T: Ownable<In>> AsMut<T> for Owned<In> {
    fn as_mut(value: &mut Self::Value) -> <T as Refable>::Mut<'_> {
        T::owned_as_mut(value)
    }
}

impl<In, T: Ownable<In>> Ref<T> for Owned<In> {
    type Ref<'a> = T::Owned;
}
