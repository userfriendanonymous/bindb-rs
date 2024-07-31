use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn output(item: &ItemStruct, lib: &syn::Path) -> super::Output {
    let (mut len_fn, mut buf_len_fn, mut encode_fn) = (quote! {}, quote! {}, quote! {});

    match &item.fields {
        syn::Fields::Unit => {
            encode_fn = quote! { 0 };
            buf_len_fn = quote! { 0 };
            len_fn = quote! { 0 };
        },
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                encode_fn = quote! {
                    #encode_fn
                    let len = #lib::dynamic::encode_ptr(#lib::BytesPtr::range_from(buf.0, cursor), &self.#ident);
                    cursor += len;
                };
                buf_len_fn = quote! {
                    #buf_len_fn
                    let len = #lib::dynamic::ptr_len::<#ty>(#lib::BytesPtr::range_from(buf.0, cursor));
                    cursor += len;
                };
                len_fn = quote! {
                    #len_fn
                    let len = <#ty as #lib::Dynamic>::len(&self.#ident);
                    cursor += len;
                };
            }
            encode_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #encode_fn
                    cursor
                }
            };
            buf_len_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #buf_len_fn
                    cursor
                }
            };
            len_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #len_fn
                    cursor
                }
            };
        }
        syn::Fields::Unnamed(fields) => {
            for (idx, field) in fields.unnamed.iter().enumerate() {
                let index = syn::Index::from(idx);
                let ty = &field.ty;
                encode_fn = quote! {
                    #encode_fn
                    let len = #lib::dynamic::encode_ptr(#lib::BytesPtr::range_from(buf.0, cursor), &self.#index);
                    cursor += len;
                };
                buf_len_fn = quote! {
                    #buf_len_fn
                    let len = #lib::dynamic::ptr_len::<#ty>(#lib::BytesPtr::range_from(buf.0, cursor));
                    cursor += len;
                };
                len_fn = quote! {
                    #len_fn
                    let len = <#ty as #lib::Dynamic>::len(&self.#index);
                    cursor += len;
                };
            }
            encode_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #encode_fn
                    cursor
                }
            };
            buf_len_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #buf_len_fn
                    cursor
                }
            };
            len_fn = quote! {
                unsafe {
                    let mut cursor = 0usize;
                    #len_fn
                    cursor
                }
            };
        },
    }

    super::Output { len_fn, buf_len_fn, encode_fn }
}
