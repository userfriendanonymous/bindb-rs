use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ItemEnum;
use crate::fixed::expand::r#impl::enum_tag_data;
use super::Output;

pub fn output(value: &ItemEnum, lib: &syn::Path) -> Output {
    let variants_len = value.variants.len();
    let (tag_size, tag_ty) = enum_tag_data(value);
    let (mut encode_fn, mut len) = (quote! {}, quote! { 0 });

    for (idx, variant) in value.variants.iter().enumerate() {
        let ident = &variant.ident;
        let (mut encode_fields, mut match_fields, mut fields_len) = (quote! {}, quote! {}, quote! { 0 });

        match &variant.fields {
            syn::Fields::Unit => {
                encode_fields = quote! {};
                match_fields = quote! {};
            },

            syn::Fields::Named(fields) => {
                encode_fields = quote! {
                    let mut cursor: usize = #tag_size;
                };
                for field in &fields.named {
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    fields_len = quote! {
                        #fields_len + <#ty as #lib::Fixed>::LEN
                    };
                    encode_fields = quote! {
                        #encode_fields
                        let len = <#ty as #lib::Fixed>::LEN;
                        #lib::fixed::encode_ptr::<#ty>(#lib::fixed::Ptr::range_at(buf.0, cursor, len), #ident);
                        cursor += len;
                    };
                    match_fields = quote! {
                        #match_fields #ident,
                    };
                }
                match_fields = quote! { { #match_fields } };
            },

            syn::Fields::Unnamed(fields) => {
                encode_fields = quote! {
                    let mut cursor: usize = #tag_size;
                };
                for (idx, field) in fields.unnamed.iter().enumerate() {
                    let ty = &field.ty;
                    let ident = syn::Ident::new(&format!("field{idx}"), Span::call_site());
                    fields_len = quote! {
                        #fields_len + <#ty as #lib::Fixed>::LEN
                    };
                    encode_fields = quote! {
                        #encode_fields
                        let len = <#ty as #lib::Fixed>::LEN;
                        #lib::fixed::encode_ptr::<#ty>(#lib::fixed::Ptr::range_at(buf.0, cursor, len), #ident);
                        cursor += len;
                    };
                    let ident = syn::Ident::new(&format!("field{idx}"), Span::call_site());
                    match_fields = quote! { #match_fields #ident, }
                }
                match_fields = quote! { ( #match_fields ) };
            }
        }

        len = quote! {
            {
                let a = #len;
                let b = #fields_len;
                [a, b][(a < b) as usize]
            }
        };
        encode_fn = quote! {
            #encode_fn
            Self::#ident #match_fields => {
                #lib::fixed::encode_ptr::<#tag_ty>(#lib::fixed::Ptr::range_at(buf.0, 0, #tag_size), &(#idx as #tag_ty));
                #encode_fields
            }
        };
    }

    len = quote! { #tag_size + #len };
    encode_fn = quote! {
        unsafe {
            match self {
                #encode_fn
            }
        }
    };
    Output { encode_fn, len }
}