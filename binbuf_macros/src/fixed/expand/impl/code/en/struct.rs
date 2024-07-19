use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;
use super::Output;

pub fn output(item: &ItemStruct, lib: &syn::Path) -> Output {
    let (mut encode_fn, mut len) = (quote! {}, quote! { 0 });

    match &item.fields {
        syn::Fields::Unit => {
            encode_fn = quote! {};
            len = quote! { 0 };
        },
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                len = quote! {
                    #len + <#ty as #lib::Fixed>::LEN
                };
                encode_fn = quote! {
                    #encode_fn
                    let len = <#ty as #lib::Fixed>::LEN;
                    #lib::fixed::encode_ptr::<#ty>(#lib::fixed::Ptr::range_at(buf.0, cursor, len), &self.#ident);
                    cursor += len;
                };
            }
            encode_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #encode_fn
                }
            };
        }
        syn::Fields::Unnamed(fields) => {
            for (idx, field) in fields.unnamed.iter().enumerate() {
                let index = syn::Index::from(idx);
                let ty = &field.ty;
                len = quote! {
                    #len + <#ty as #lib::Fixed>::LEN
                };
                encode_fn = quote! {
                    #encode_fn
                    let len = <#ty as #lib::Fixed>::LEN;
                    #lib::fixed::encode_ptr::<#ty>(#lib::fixed::Ptr::range_at(buf.0, cursor, len), &self.#index);
                    cursor += len;
                };
            }
            encode_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #encode_fn
                }
            };
        },
    }

    Output { encode_fn, len }
}