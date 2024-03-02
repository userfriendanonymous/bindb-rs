pub struct BaseFull<O: Instance<Base>>(O::Value);

pub struct Base;

impl Ownable<Owned1> for Base {
    fn owned_as_mut<'a>(value: &mut Owned1) -> Self::Mut<'a> {
        todo!()
    }

    fn owned_as_ref<'a>(value: &Owned1) -> Self::Const<'a> {
        todo!()
    }
}

pub struct Owned1;
pub struct Owned2;

trait TypeFn<In> {
    type Out;
}

type Apply<In, F: TypeFn<In>> = F::Out;

pub struct OwnerShip<T, O>(PhantomData<(T, O)>);

// type_fn! {
//     <'a, T> for OwnerShip<T> =
//         Const<'a> -> T::Const<'a>,
//         Mut<'a> -> T::Mut<'a>,
//         Owned -> T::Owned,
// }
//
mod own {}

impl<O> Owned1 where OwnerShip<Self, Owned2>: TypeFn<O> {}

impl<'a, O, T: Refable> TypeFn<Const<'a>> for OwnerShip<T, O> {
    type Out = T::Const<'a>;
}

impl<'a, O, T: Ownable<O>> TypeFn<Owned> for OwnerShip<T, O> {
    type Out = O;
}

fn idk<'a>() {
    let wow: Apply<Owned, OwnerShip<u32, Owned1>> = todo!();
}
