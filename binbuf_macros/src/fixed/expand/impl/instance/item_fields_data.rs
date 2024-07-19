use proc_macro2::TokenStream;
use quote::quote;

use crate::fixed::{expand::r#impl::enum_tag_data, Item};

use super::LensFieldAttr;

pub struct Value {
    pub lens_fns: TokenStream,
    pub is_external: bool,
}

pub fn get(item: &Item, lib: &syn::Path) -> Value {
    let mut lens_fns = quote! {};

    match item {
        Item::Struct(item) => {
            let mut len = quote! { 0 };
            match &item.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        let ty = &field.ty;
                        for attr in &field.attrs {
                            match &attr.meta {
                                syn::Meta::List(meta) => {
                                    match meta.path.require_ident().unwrap().to_string().as_str() {
                                        "lens" => {
                                            let LensFieldAttr { vis, fn_ident } = syn::parse(meta.tokens.clone().into())
                                                .expect("failed to parse lens attribute tokens");
                                            lens_fns = quote! {
                                                #lens_fns
                                                #vis fn #fn_ident<P: #lib::fixed::Ptr>(buf: #lib::fixed::Buf<Self, P>) -> #lib::fixed::Buf<#ty, P> {
                                                    unsafe {
                                                        <#ty as #lib::Entry>::buf(#lib::fixed::Ptr::range_at(buf.0, #len, <#ty as #lib::Fixed>::LEN))
                                                    }
                                                }
                                            }
                                        },
                                        _ => panic!("unexpected attribute")
                                    }
                                },
                                _ => {}
                            }
                        }
                        len = quote! {
                            #len + <#ty as #lib::Fixed>::LEN
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &fields.unnamed {
                        let ty = &field.ty;
                        len = quote! {
                            #len + <#ty as #lib::Fixed>::LEN
                        }
                    }
                }
                syn::Fields::Unit => {}
            }
            
            Value {
                lens_fns,
                is_external: false,
            }
        }
        Item::Enum(item) => {
            let (tag_size, _tag_ty) = enum_tag_data(item);
            let mut len = quote! { 0 };
            //panic!("VARIANTS LEN: {}", &item.variants.len());
            for variant in item.variants.iter() {
                let mut fields_len = quote! { 0 };
                for field in variant.fields.iter() {
                    let ty = &field.ty;
                    fields_len = quote! {
                        #fields_len + <#ty as #lib::Fixed>::LEN
                    };
                }
                len = quote! {
                    {
                        let a = #len;
                        let b = #fields_len;
                        [a, b][(a < b) as usize]
                    }
                };
            }
            len = quote! { #tag_size + #len };
    
            Value {
                lens_fns: quote! {},
                is_external: false,
            }
        },
    }
}