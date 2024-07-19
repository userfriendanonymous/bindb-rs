use std::collections::BTreeMap;
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::fixed::Item;

pub fn output(
    value: &syn::ItemImpl,
    items: &BTreeMap<String, Item>,
    lib: &syn::Path,
) -> TokenStream {
    let item = items.get(&value.self_ty.to_token_stream().to_string()).unwrap();

}
