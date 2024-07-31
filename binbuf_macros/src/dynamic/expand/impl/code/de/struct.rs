use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn output(item: &ItemStruct, lib: &syn::Path) -> TokenStream {
    let decode_fn;

    match &item.fields {
        syn::Fields::Unit => {
            decode_fn = quote! {
                Self
            };
        },
        syn::Fields::Named(fields) => {
            let iter = fields.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote! {
                    #ident: {
                        let (v, len) = <#ty as #lib::dynamic::Decode>::decode(<#ty as #lib::Entry>::buf(unsafe { #lib::dynamic::Ptr::range_from(buf.0, cursor) }));
                        cursor += len;
                        v
                    }
                }
            });
            decode_fn = quote! {
                unsafe {
                    let mut cursor: usize = 0;
                    let s = Self {
                        #( #iter ),*
                    };
                    (s, cursor)
                }
            };
        }
        syn::Fields::Unnamed(fields) => {
            let iter = fields.unnamed.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    {
                        let (v, len) = <#ty as #lib::dynamic::Decode>::decode(<#ty as #lib::Entry>::buf(unsafe { #lib::dynamic::Ptr::range_from(buf.0, cursor) }));
                        cursor += len;
                        v
                    }
                }
            });
            decode_fn = quote! {
                unsafe {
                    let mut cursor: usize = 0;
                    let s = Self (
                        #( #iter ),*
                    );
                    (s, cursor)
                }
            };
        },
    }

    decode_fn
}