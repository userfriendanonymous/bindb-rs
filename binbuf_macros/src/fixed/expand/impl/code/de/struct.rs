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
                        let len = <#ty as #lib::Fixed>::LEN;
                        let v = #lib::fixed::decode_ptr::<#ty>(#lib::fixed::Ptr::range_at(buf.0, cursor, len));
                        cursor += len;
                        v
                    }
                }
            });
            decode_fn = quote! {
                unsafe {
                    let mut cursor: usize = 0;
                    Self {
                        #( #iter ),*
                    }
                }
            };
        }
        syn::Fields::Unnamed(fields) => {
            let iter = fields.unnamed.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    {
                        let len = <#ty as #lib::Fixed>::LEN;
                        let v = #lib::fixed::decode_ptr::<#ty>(#lib::fixed::Ptr::range_at(buf.0, cursor, len));
                        cursor += len;
                        v
                    }
                }
            });
            decode_fn = quote! {
                unsafe {
                    let mut cursor: usize = 0;
                    Self (
                        #( #iter ),*
                    )
                }
            };
        },
    }

    decode_fn
}