use proc_macro2::TokenStream;

use crate::dynamic::Item;

pub mod r#enum;
pub mod r#struct;

pub struct Output {
    pub len_fn: TokenStream,
    pub buf_len_fn: TokenStream,
    pub encode_fn: TokenStream,
}

pub fn output(item: &Item, lib: &syn::Path) -> Output {
    match item {
        Item::Struct(item) => {
            r#struct::output(item, lib)
        }
        Item::Enum(value) => {
            r#enum::output(value, lib);
            todo!()
        }
    }
}