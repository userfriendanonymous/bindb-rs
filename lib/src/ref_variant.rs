use std::marker::PhantomData;

pub type Value<V: Instance, T> = V::Value<T>;

pub trait Instance {
    type Value<T>;
}

pub trait Ref: Instance {
    type Ref<'a>;
}

pub trait AsConst: Instance {
    fn as_const<'a, T>(value: &'a Self::Value<T>) -> &'a T;
}

pub trait AsMut: Instance {
    fn as_mut<'a, T>(value: &'a mut Self::Value<T>) -> &'a mut T;
}

pub struct Const<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Const<'a> {
    type Value<T> = &'a T;
}

impl<'b> Ref for Const<'b> {
    type Ref<'a> = Const<'a>;
}

impl<'b> AsConst for Const<'b> {
    fn as_const<'a, T>(value: &'a Self::Value<T>) -> <Const<'a> as Instance>::Value<T> {
        *value
    }
}

pub struct Mut<'a>(PhantomData<&'a ()>);

impl<'a> Instance for Mut<'a> {
    type Value<T> = &'a mut T;
}

impl<'b> Ref for Mut<'b> {
    type Ref<'a> = Mut<'a>;
}

impl<'b> AsConst for Mut<'b> {
    fn as_const<'a, T>(value: &'a Self::Value<T>) -> <Const<'a> as Instance>::Value<T> {
        *value
    }
}

impl<'b> AsMut for Mut<'b> {
    fn as_mut<'a, T>(value: &'a mut Self::Value<T>) -> <Mut<'a> as Instance>::Value<T> {
        *value
    }
}

pub struct Owned;

impl Instance for Owned {
    type Value<T> = T;
}

impl AsConst for Owned {
    fn as_const<'a, T>(value: &'a Self::Value<T>) -> <Const<'a> as Instance>::Value<T> {
        value
    }
}

impl AsMut for Owned {
    fn as_mut<'a, T>(value: &'a mut Self::Value<T>) -> <Mut<'a> as Instance>::Value<T> {
        value
    }
}