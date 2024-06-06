use super::Item;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::parse::Parser as _;

pub fn output(
    value: syn::ItemImpl,
    items: &BTreeMap<String, Item>,
    lib: &syn::Path,
) -> TokenStream {
    let bv_trait_bound: syn::TraitBound =
        syn::parse2(quote! { #lib::entry::bytes::Variant }).unwrap();

    let self_ty = *value.self_ty;
    let (impl_generics, ty_generics, where_clause) = value.generics.split_for_impl();

    let item: &Item = match self_ty.clone() {
        syn::Type::Path(mut value) => {
            value.path.segments.last_mut().unwrap()
                .arguments = syn::PathArguments::None;
            items
                .get(&value.path.into_token_stream().to_string())
        },
        _ => None,
    }.expect("Type must be inside this entry! macro");

    let encode_fn;
    let decode_fn;


    match item {
        Item::Struct(item) => match &item.fields {
            syn::Fields::Unit => {
                encode_fn = quote! {};
                decode_fn = quote! {
                    Self
                };
            },
            syn::Fields::Named(fields) => {
                let iter = fields.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        let len = <#ty as #lib::Entry>::len();
                        <#ty as #lib::entry::Codable>::encode(&self.#ident, <#ty as #lib::Entry>::buf(unsafe { buf.0.index_range(cursor, len) }));
                        cursor += len;
                    }
                });
                encode_fn = quote! {
                    let mut cursor = 0usize;
                    #( #iter )*
                };

                let iter = fields.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        #ident: {
                            let len = <#ty as #lib::Entry>::len();
                            let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { buf.0.index_range(cursor, len) }));
                            cursor += len;
                            v
                        }
                    }
                });
                decode_fn = quote! {
                    let mut cursor: usize = 0;
                    Self {
                        #( #iter ),*
                    }
                };
            }
            syn::Fields::Unnamed(fields) => {
                let iter = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                    // let ident = field.ident.as_ref().unwrap();
                    let index = syn::Index::from(idx);
                    let ty = &field.ty;
                    quote! {
                        let len = <#ty as #lib::Entry>::len();
                        <#ty as #lib::entry::Codable>::encode(&self.#index, <#ty as #lib::Entry>::buf(unsafe { buf.0.index_range(cursor, len) }));
                        cursor += len;
                    }
                });
                encode_fn = quote! {
                    let mut cursor = 0usize;
                    #( #iter )*
                };

                let iter = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;
                    quote! {
                        {
                            let len = <#ty as #lib::Entry>::len();
                            let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { buf.0.index_range(cursor, len) }));
                            cursor += len;
                            v
                        }
                    }
                });
                decode_fn = quote! {
                    let mut cursor: usize = 0;
                    Self (
                        #( #iter ),*
                    )
                };
            },
        },
        Item::Enum(_) => todo!()
    }

    quote! {
        impl #impl_generics #lib::entry::Codable for #self_ty #where_clause {
            fn encode(&self, buf: #lib::entry::BufMut<'_, Self>) {
                #encode_fn
            }
            fn decode(buf: #lib::entry::BufConst<'_, Self>) -> Self {
                #decode_fn
            }
        }
    }
}

// fn encode(&self, buf: BufMut<'_, Self>);
// fn decode(buf: BufConst<'_, Self>) -> Self;
// lens! { struct HeaderMetaLens(meta); }
// struct Header<M, T> {
//      #[lens(pub field_meta)]
//      meta: M
//      #[lens(pub field_id)]
//      id: entry::Id<T>
// }
