use std::collections::BTreeMap;
use proc_macro2::TokenStream;
use super::super::Item;

mod instance;
mod codable;

pub fn output(value: syn::ItemImpl, items: &BTreeMap<syn::Ident, Item>, lib: &syn::Path) -> TokenStream {
    match value.trait_.clone().unwrap().1.require_ident().unwrap().to_string().as_str() {
        "I" => instance::output(value.clone(), items, lib),
        "Codable" => {
            todo!()
        },
        _ => panic!("No such impl expected")
    }
}

// const LEN: usize;
// type Buf<BV: bytes::Variant>;
// fn buf<BV: bytes::Variant>(bytes: Bytes<BV, { Self::LEN }>) -> Self::Buf<BV>;
// fn buf_rb_const<'a>(buf: &'a BufConst<'a, Self>) -> BufConst<'a, Self>;
// fn buf_rb_mut<'a>(buf: &'a mut BufMut<'a, Self>) -> BufMut<'a, Self>;
// fn buf_owned_as_const(buf: &BufOwned<Self>) -> BufConst<'_, Self>;
// fn buf_owned_as_mut(buf: &mut BufOwned<Self>) -> BufMut<'_, Self>;
// fn buf_mut_as_const<'a>(buf: &'a BufMut<'a, Self>) -> BufConst<'a, Self>;
// unsafe fn buf_detach<'b, BV: bytes::variant::Ref>(buf: Self::Buf<BV>) -> Self::Buf<BV::Ref<'b>>;
// fn buf_copy_to(src: BufConst<'_, Self>, dst: BufMut<'_, Self>);
// unsafe fn buf_copy_nonoverlapping_to() {}
// fn buf_swap(a: BufMut<'_, Self>, b: BufMut<'_, Self>);