use super::Item;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::parse::Parser as _;

pub fn output(
    value: syn::ItemImpl,
    items: &BTreeMap<String, Item>,
    lib: &syn::Path,
) -> TokenStream {
    let bv_trait_bound: syn::TraitBound =
        syn::parse2(quote! { #lib::entry::Ptr }).unwrap();

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
                        {
                            let len = <#ty as #lib::Entry>::LEN;
                            <#ty as #lib::entry::Codable>::encode(&self.#ident, <#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, cursor, len) }));
                            cursor += len;
                        }
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
                            let len = <#ty as #lib::Entry>::LEN;
                            let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, cursor, len) }));
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
                        {
                            let len = <#ty as #lib::Entry>::LEN;
                            <#ty as #lib::entry::Codable>::encode(&self.#index, <#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, cursor, len) }));
                            cursor += len;
                        }
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
                            let len = <#ty as #lib::Entry>::LEN;
                            let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, cursor, len) }));
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
        Item::Enum(value) => {
            let variants_len = value.variants.len();
            let (tag_size, tag_ty) = super::enum_tag_data(value);

            let branches = value.variants.iter().enumerate().map(|(idx, variant)| {
                let ident = &variant.ident;
                let encode_fields;
                let match_fields;
                match &variant.fields {
                    syn::Fields::Unit => {
                        encode_fields = quote! {};
                        match_fields = quote! {};
                    },
                    syn::Fields::Named(fields) => {
                        let iter = fields.named.iter().map(|field| {
                            let ident = field.ident.as_ref().unwrap();
                            let ty = &field.ty;
                            quote! {
                                let len = <#ty as #lib::Entry>::LEN;
                                <#ty as #lib::entry::Codable>::encode(#ident, <#ty as #lib::Entry>::buf(#lib::entry::Ptr::index_range(buf.0, cursor, len)));
                                cursor += len;
                            }
                        });
                        encode_fields = quote! {
                            let mut cursor: usize = #tag_size;
                            #( #iter )*
                        };

                        let iter = fields.named.iter().map(|field| {
                            let ident = field.ident.as_ref().unwrap();
                            quote! { #ident }
                        });
                        match_fields = quote! {
                            { #( #iter ),* }
                        };
                    },
                    syn::Fields::Unnamed(fields) => {
                        let iter = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                            let ty = &field.ty;
                            let ident = syn::Ident::new(&format!("field{idx}"), Span::call_site());
                            quote! {
                                let len = <#ty as #lib::Entry>::LEN;
                                <#ty as #lib::entry::Codable>::encode(#ident, <#ty as #lib::Entry>::buf(#lib::entry::Ptr::index_range(buf.0, cursor, len)));
                                cursor += len;
                            }
                        });
                        encode_fields = quote! {
                            let mut cursor: usize = #tag_size;
                            #( #iter )*
                        };

                        let iter = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                            let ident = syn::Ident::new(&format!("field{idx}"), Span::call_site());
                            quote! { #ident }
                        });
                        match_fields = quote! {
                            ( #( #iter ),* )
                        };
                    }
                }
                quote! {
                    Self::#ident #match_fields => {
                        <#tag_ty as #lib::entry::Codable>::encode(
                            &(#idx as #tag_ty),
                            <#tag_ty as #lib::Entry>::buf(#lib::entry::Ptr::index_range(buf.0, 0, #tag_size))
                        );
                        #encode_fields
                    }
                }
            });
            encode_fn = quote! {
                unsafe {
                    match self {
                        #( #branches ),*
                    }
                }
            };

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
                                    let len = <#ty as #lib::Entry>::LEN;
                                    let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, cursor, len) }));
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
                                    let len = <#ty as #lib::Entry>::LEN;
                                    let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, cursor, len) }));
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
                    let idx = <#tag_ty as #lib::entry::Codable>::decode(
                        <#tag_ty as #lib::Entry>::buf(#lib::entry::Ptr::index_range(buf.0, 0, #tag_size))
                    ) % (#variants_len as #tag_ty);
                    match idx {
                        #( #branches ),*
                        _ => ::std::unreachable!()
                    }
                }
            };
        }
    }

    quote! {
        impl #impl_generics #lib::entry::Codable for #self_ty #where_clause {
            fn encode(&self, mut buf: #lib::entry::BufMut<Self>) {
                #encode_fn
            }
            fn decode(buf: #lib::entry::BufConst<Self>) -> Self {
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
