use std::collections::BTreeMap;
use proc_macro2::TokenStream;
use super::super::Item;

mod instance;
mod codable;

pub fn output(value: syn::ItemImpl, items: &BTreeMap<String, Item>, lib: &syn::Path) -> TokenStream {
    match value.trait_.clone().unwrap().1.require_ident().unwrap().to_string().as_str() {
        "I" => instance::output(value.clone(), items, lib),
        "Codable" => codable::output(value.clone(), items, lib),
        _ => panic!("No such impl expected")
    }
}
