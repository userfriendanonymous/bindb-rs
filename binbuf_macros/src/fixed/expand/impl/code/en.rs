use proc_macro2::TokenStream;

use crate::fixed::Item;

pub mod r#enum;
pub mod r#struct;

pub struct Output {
    pub encode_fn: TokenStream,
    pub len: TokenStream,
}

pub fn output(item: &Item, lib: &syn::Path) -> Output {
    match item {
        Item::Struct(item) => {
            r#struct::output(item, lib)
        }
        Item::Enum(value) => {
            r#enum::output(value, lib)
        }
    }
}