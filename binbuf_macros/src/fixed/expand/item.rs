use proc_macro2::TokenStream;
use quote::quote;
use super::super::Item;

pub fn output(value: Item, _lib: &syn::Path) -> TokenStream {
    match value {
        Item::Struct(mut value) => {
            match &mut value.fields {
                syn::Fields::Named(fields) => {
                    for field in fields.named.iter_mut() {
                        field.attrs.clear();
                    }
                },
                syn::Fields::Unnamed(fields) => {
                    for field in fields.unnamed.iter_mut() {
                        field.attrs.clear();
                    }
                },
                syn::Fields::Unit => {},
            }
            quote! { #value }
        },
        Item::Enum(value) => quote! { #value }
    }
}
