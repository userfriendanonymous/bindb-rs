use super::Item;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{collections::BTreeMap, marker::PhantomData};

mod item_fields_data;

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

pub struct ImplInput {
    buf: syn::ImplItemType,
}

impl ImplInput {
    pub fn get(items: Vec<syn::ImplItem>) -> Self {
        let mut buf = None;
        for item in items {
            match item {
                syn::ImplItem::Type(item) => match item.ident.to_string().as_str() {
                    "Buf" => buf = Some(item),
                    _ => panic!("No such type item expected"),
                },
                _ => panic!("No such item expected"),
            }
        }
        Self {
            buf: buf.expect("Buf expected"),
        }
    }
}

pub fn output(
    value: syn::ItemImpl,
    items: &BTreeMap<String, Item>,
    lib: &syn::Path,
) -> TokenStream {
    let ptr_trait_bound: syn::TraitBound =
        syn::parse2(quote! { #lib::fixed::Ptr }).unwrap();

    let self_ty = *value.self_ty;
    let (impl_generics, _ty_generics, where_clause) = value.generics.split_for_impl();

    let impl_input = ImplInput::get(value.items);

    let item_fields_data = match self_ty.clone() {
        syn::Type::Path(mut value) => {
            value.path.segments.last_mut().unwrap()
                .arguments = syn::PathArguments::None;
            items
                .get(&value.path.into_token_stream().to_string())
                .map(|item| item_fields_data::get(item, lib))
        },
        _ => None,
    }
    .unwrap_or_else(|| item_fields_data::Value {
        lens_fns: quote! { },
        is_external: true,
    });
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

            unsafe fn buf<P: #lib::fixed::Ptr>(ptr: P) -> Self::Buf<P> {
                #buf_path(ptr, ::std::marker::PhantomData)
            }
            fn buf_ptr<P: #lib::fixed::Ptr>(buf: Self::Buf<P>) -> P {
                buf.0
            }
        }
    }
}
