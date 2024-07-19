use std::collections::BTreeMap;
use proc_macro2::TokenStream;
use super::super::Item;
use quote::quote;

mod instance;
mod code;
// mod buf_eq;

fn enum_tag_data(value: &syn::ItemEnum) -> (usize, TokenStream) {
    match value.variants.len() {
        x if x < 256 => (1, quote! { ::std::primitive::u8 }),
        _ => panic!("Not yet supported")
    }
}

pub fn output(value: syn::ItemImpl, items: &BTreeMap<String, Item>, lib: &syn::Path) -> TokenStream {
    match value.trait_.clone().unwrap().1.require_ident().unwrap().to_string().as_str() {
        "I" => instance::output(value.clone(), items, lib),
        "Code" => code::output(value.clone(), code::State::Code, items, lib),
        "Encode" => code::output(value.clone(), code::State::Encode, items, lib),
        "Decode" => code::output(value.clone(), code::State::Decode, items, lib),
        // "BufEq" => buf_eq::output(value.clone(), false, items, lib),
        _ => panic!("No such impl expected")
    }
}
