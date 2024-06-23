use super::Item;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::{parse::Parser as _, parse_macro_input};

struct LensFieldAttr {
    vis: syn::Visibility,
    fn_ident: syn::Ident,
}

impl syn::parse::Parse for LensFieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let fn_ident = input.parse()?;
        Ok(Self {
            vis,
            fn_ident
        })
    }
}

pub struct ItemFieldsData {
    len: TokenStream,
    lens_fns: TokenStream,
    is_external: bool,
}

pub fn item_fields_data(item: &Item, lib: &syn::Path) -> ItemFieldsData {
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
                                            let LensFieldAttr { vis, fn_ident } = syn::parse(meta.tokens.clone().into()).expect("failed to parse lens attribute tokens");
                                            lens_fns = quote! {
                                                #lens_fns
                                                #vis fn #fn_ident<P: #lib::entry::Ptr>(buf: #lib::entry::Buf<Self, P>) -> #lib::entry::Buf<#ty, P> {
                                                    <#ty as #lib::Entry>::buf(unsafe { #lib::entry::Ptr::index_range(buf.0, #len, <#ty as #lib::Entry>::LEN) })
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
                            #len + <#ty as #lib::Entry>::LEN
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &fields.unnamed {
                        let ty = &field.ty;
                        len = quote! {
                            #len + <#ty as #lib::Entry>::LEN
                        }
                    }
                }
                syn::Fields::Unit => {}
            }
            ItemFieldsData {
                len,
                lens_fns,
                is_external: false,
            }
        }
        Item::Enum(_) => todo!(),
    }
}

pub struct ImplInput {
    buf: syn::ImplItemType,
    len: Option<TokenStream>,
}

impl ImplInput {
    pub fn get(items: Vec<syn::ImplItem>) -> Self {
        let (mut buf, mut len) = (None, None);
        for item in items {
            match item {
                syn::ImplItem::Const(item) => match item.ident.to_string().as_str() {
                    "LEN" => {
                        let expr = item.expr;
                        len = Some(quote! { #expr });
                    },
                    _ => panic!("No such const item expected")
                }
                syn::ImplItem::Type(item) => match item.ident.to_string().as_str() {
                    "Buf" => buf = Some(item),
                    _ => panic!("No such type item expected"),
                },
                _ => panic!("No such item expected"),
            }
        }
        Self {
            buf: buf.expect("Buf expected"),
            len,
        }
    }
}

pub fn output(
    value: syn::ItemImpl,
    items: &BTreeMap<String, Item>,
    lib: &syn::Path,
) -> TokenStream {
    let ptr_trait_bound: syn::TraitBound =
        syn::parse2(quote! { #lib::entry::Ptr }).unwrap();

    let self_ty = *value.self_ty;
    let (impl_generics, ty_generics, where_clause) = value.generics.split_for_impl();

    let impl_input = ImplInput::get(value.items);

    let item_fields_data = match self_ty.clone() {
        syn::Type::Path(mut value) => {
            value.path.segments.last_mut().unwrap()
                .arguments = syn::PathArguments::None;
            items
                .get(&value.path.into_token_stream().to_string())
                .map(|item| item_fields_data(item, lib))
        },
        _ => None,
    }
    .or_else(|| impl_input.len.map(|len| ItemFieldsData {
        len,
        lens_fns: quote! { },
        is_external: true,
    }))
    .expect("Can't derive LEN");
    let len = item_fields_data.len;
    let lens_fns = item_fields_data.lens_fns;

    let buf_ty = impl_input.buf.ty;
    let buf_path = match buf_ty.clone() {
        syn::Type::Path(path) => {
            let mut path = path.path;
            path.segments
                .last_mut()
                .expect("Buf path has no segments")
                .arguments = syn::PathArguments::None;
            path
        }
        _ => panic!("Path expected"),
    };

    let mut buf_generics = impl_input.buf.generics;
    let bv_param: &mut syn::TypeParam = &mut buf_generics.type_params_mut().next().unwrap();
    bv_param
        .bounds
        .push(syn::TypeParamBound::Trait(ptr_trait_bound));

    let item_impl = if item_fields_data.is_external {
        quote! {
            
        }
    } else {
        quote! {
            impl #impl_generics #self_ty #where_clause {
                #lens_fns
            }
        }
    };

    quote! {
        #item_impl

        impl #impl_generics #lib::Entry for #self_ty #where_clause {
            type Buf #buf_generics = #buf_ty;

            const LEN: usize = #len;
            // fn len() -> usize {
            //     #len
            // }

            fn buf<P: #lib::entry::Ptr>(ptr: P) -> Self::Buf<P> {
                #buf_path(ptr, std::marker::PhantomData)
            }
            fn buf_ptr<P: #lib::entry::Ptr>(buf: Self::Buf<P>) -> P {
                buf.0
            }
        }
    }
}

