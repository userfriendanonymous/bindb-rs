use super::Item;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeMap;

mod en;
mod de;

#[derive(Clone, Copy)]
pub enum State {
    Decode,
    Encode,
    Code,
}

pub fn output(
    value: syn::ItemImpl,
    state: State,
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

    let (is_encode, is_decode) = match state {
        State::Code => (true, true),
        State::Decode => (false, true),
        State::Encode => (true, false)
    };

    let encode_impl = is_encode.then(|| {
        let en::Output { encode_fn, len } = en::output(item, lib);
        quote! {
            impl #impl_generics #lib::Fixed for #self_ty #where_clause {
                const LEN: usize = #len;
                fn encode(&self, mut buf: #lib::fixed::BufMut<Self>) {
                    #encode_fn
                }
            }
        }
    });

    let decode_impl = is_decode.then(|| {
        let decode_fn = de::output(item, lib);
        quote! {
            impl #impl_generics #lib::fixed::Decode for #self_ty #where_clause {
                fn decode(buf: #lib::fixed::BufConst<Self>) -> Self {
                    #decode_fn
                }
            }
        }
    });
    
    quote! {
        #encode_impl
        #decode_impl
    }
}
