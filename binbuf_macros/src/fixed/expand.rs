use proc_macro2::TokenStream;
use quote::quote;
use super::Input;

pub mod r#impl;
pub mod item;
pub mod r#macro;

pub fn output(input: Input, lib: &syn::Path) -> TokenStream {
    let mut output = quote! { };

    for value in input.impls {
        let more_output = r#impl::output(value, &input.items, lib);
        output = quote! { #output #more_output };
    }

    for value in input.items.values() {
        let more_output = item::output(value.clone(), lib);
        output = quote! { #output #more_output };
    }

    for value in input.macros {
        let more_output = r#macro::output(value, &input.items, lib);
        output = quote! { #output #more_output };
    }

    output
}