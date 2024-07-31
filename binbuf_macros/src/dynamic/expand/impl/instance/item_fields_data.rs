use proc_macro2::TokenStream;
use quote::quote;

use crate::dynamic::{expand::r#impl::enum_tag_data, Item};

use super::LensFieldAttr;

pub struct Value {
    pub lens_fns: TokenStream,
    pub is_external: bool,
}

pub fn get(item: &Item, lib: &syn::Path) -> Value {
    let mut lens_fns = quote! {};

    match item {
        Item::Struct(item) => {
            match &item.fields {
                syn::Fields::Named(fields) => {
                    let mut len_fn = quote! { };

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
                                                #vis fn #fn_ident<P: #lib::dynamic::Ptr>(buf: #lib::dynamic::Buf<Self, P>) -> #lib::dynamic::Buf<#ty, P> {
                                                    let mut cursor = 0;
                                                    #len_fn
                                                    unsafe { <#ty as #lib::Entry>::buf(#lib::dynamic::Ptr::range_from(buf.0, cursor)) }
                                                }
                                            }
                                        },
                                        _ => panic!("unexpected attribute")
                                    }
                                },
                                _ => {}
                            }
                        }

                        len_fn = quote! {
                            #len_fn
                            cursor += #lib::dynamic::ptr_len(#lib::dynamic::Ptr::range_from(buf.0, cursor));
                        };
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &fields.unnamed {
                        let _ty = &field.ty;
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
            Value {
                lens_fns: quote! {},
                is_external: false,
            }
        },
    }
}