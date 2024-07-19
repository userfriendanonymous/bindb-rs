use proc_macro2::TokenStream;

use crate::fixed::Item;

pub mod r#enum;
pub mod r#struct;

pub fn output(item: &Item, lib: &syn::Path) -> TokenStream {
    match item {
        Item::Struct(item) => {
            r#struct::output(item, lib)
        }
        Item::Enum(value) => {
            r#enum::output(value, lib)
        }
    }
}