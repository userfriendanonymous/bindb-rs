use proc_macro2::TokenStream;
use quote::quote;
use crate::entry::Item;


pub fn output(value: &Item, lib: &syn::Path) -> TokenStream {
    match value {
        Item::Struct(value) => quote! { value },
        Item::Enum(value) => quote! { value }
    }
}
