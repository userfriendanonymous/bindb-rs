use crate::{ref_variant, RefVariant};

pub type Mut<'a, V: super::Variant, const L: usize> = Value<ref_variant::Mut<'a>, V, L>;
pub type Const<'a, V: super::Variant, const L: usize> = Value<ref_variant::Const<'a>, V, L>;
pub type Owned<V: super::Variant, const L: usize> = Value<ref_variant::Owned, V, L>;

pub struct Value<RV: RefVariant, V: super::Variant, const L: usize>(RV::Value<super::Value<V, L>>);

impl<RV: RefVariant, V: super::Variant, const L: usize> Value<RV, V, L> {
    pub fn new(value: RV::Value<Value<V, L>>) -> Self {
        Self(value)
    }

    pub(crate) fn get(self) -> RV::Value<Value<V, L>> {
        self.0
    }
}