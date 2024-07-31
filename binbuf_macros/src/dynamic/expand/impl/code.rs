use super::Item;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;

mod en;
mod de;

pub fn output(
    value: syn::ItemImpl,
    is_decode: bool,
    items: &BTreeMap<String, Item>,
    lib: &syn::Path,
) -> TokenStream {
    let _bv_trait_bound: syn::TraitBound =
        syn::parse2(quote! { #lib::entry::Ptr }).unwrap();

    let self_ty = *value.self_ty;
    let (impl_generics, _ty_generics, where_clause) = value.generics.split_for_impl();

    let item: &Item = match self_ty.clone() {
        syn::Type::Path(mut value) => {
            value.path.segments.last_mut().unwrap()
                .arguments = syn::PathArguments::None;
            items
                .get(&value.path.into_token_stream().to_string())
        },
        _ => None,
    }.expect("Type must be inside this entry! macro");

    let en::Output { encode_fn, len_fn, buf_len_fn } = en::output(item, lib);

    let decode_impl = is_decode.then(|| {
        let decode_fn = de::output(item, lib);
        quote! {
            impl #impl_generics #lib::dynamic::Decode for #self_ty #where_clause {
                fn decode(buf: #lib::dynamic::BufConst<Self>) -> (Self, usize) {
                    #decode_fn
                }
            }
        }
    });

    quote! {
        impl #impl_generics #lib::Dynamic for #self_ty #where_clause {
            fn encode(&self, mut buf: #lib::dynamic::BufMut<Self>) -> usize {
                #encode_fn
            }
            fn buf_len(buf: #lib::dynamic::BufConst<Self>) -> usize {
                #buf_len_fn
            }
            fn len(&self) -> usize {
                #len_fn
            }
        }

        #decode_impl
    }
}
