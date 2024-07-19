use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::ItemEnum;
use crate::dynamic::expand::r#impl::enum_tag_data;

pub fn output(value: &ItemEnum, lib: &syn::Path) -> TokenStream {
    let variants_len = value.variants.len();
    let (tag_size, tag_ty) = enum_tag_data(value);
    let decode_fn;

    let branches = value.variants.iter().enumerate().map(|(idx, variant)| {
        let idx = idx as u8;
        let variant_ident = &variant.ident;
        let decode_fields;
        match &variant.fields {
            syn::Fields::Unit => {
                decode_fields = quote! { Self::#variant_ident };
            },
            syn::Fields::Named(fields) => {
                let iter = fields.named.iter().map(|field| {
                    let ident = &field.ident;
                    let ty = &field.ty;
                    quote! {
                        #ident: {
                            let len = <#ty as #lib::Fixed>::LEN;
                            let v = <#ty as #lib::fixed::Codable>::decode(<#ty as #lib::Fixed>::buf(unsafe { #lib::fixed::Ptr::index_range(buf.0, cursor, len) }));
                            cursor += len;
                            v
                        }
                    }
                });
                decode_fields = quote! {
                    let mut cursor: usize = #tag_size;
                    Self::#variant_ident { #( #iter ),* }
                };
            },
            syn::Fields::Unnamed(fields) => {
                let iter = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! {
                        {
                            let len = <#ty as #lib::Fixed>::LEN;
                            let v = <#ty as #lib::fixed::Codable>::decode(<#ty as #lib::Fixed>::buf(#lib::fixed::Ptr::index_range(buf.0, cursor, len)));
                            cursor += len;
                            v
                        }
                    }
                });
                decode_fields = quote! {
                    let mut cursor: usize = #tag_size;
                    Self::#variant_ident ( #( #iter ),* )
                };
            }
        }
        quote! {
            #idx => {
                #decode_fields
            }
        }
    });
    decode_fn = quote! {
        unsafe {
            let idx = <#tag_ty as #lib::fixed::Codable>::decode(
                <#tag_ty as #lib::Fixed>::buf(#lib::fixed::Ptr::index_range(buf.0, 0, #tag_size))
            ) % (#variants_len as #tag_ty);
            match idx {
                #( #branches ),*
                _ => ::std::unreachable!()
            }
        }
    };
    decode_fn
}