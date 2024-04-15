use std::collections::BTreeMap;
use super::Item;
use proc_macro2::TokenStream;
use quote::quote;

pub fn item_len(item: &Item, lib: &syn::Path) -> TokenStream {
    match item {
        Item::Struct(item) => {
            let mut len = quote! { 0 };
            match &item.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        let ty = &field.ty;
                        len = quote! {
                            #len + <#ty as #lib::Entry>::LEN
                        }
                    }
                },
                syn::Fields::Unnamed(fields) => {
                    for field in &fields.unnamed {
                        let ty = &field.ty;
                        len = quote! {
                            #len + <#ty as #lib::Entry>::LEN
                        }
                    }
                },
                syn::Fields::Unit => {}
            }
            len
        },
        Item::Enum(_) => todo!(),
    }
}

pub struct ImplInput {
    buf: syn::ImplItemType,
    len: Option<TokenStream>
}

impl ImplInput {
    pub fn get(items: Vec<syn::ImplItem>) -> Self {
        let (mut buf, mut len) = (None, None);
        for item in items {
            match item {
                syn::ImplItem::Const(item) => {
                    match item.ident.to_string().as_str() {
                        "LEN" => {
                            let expr = item.expr;
                            len = Some(quote! { #expr })
                        },
                        _ => panic!("No such const item expected")
                    }
                },
                syn::ImplItem::Type(item) => {
                    match item.ident.to_string().as_str() {
                        "Buf" => buf = Some(item),
                        _ => panic!("No such type item expected")
                    }
                },
                _ => panic!("No such item expected")
            }
        }
        Self {
            buf: buf.unwrap(),
            len,
        }
    }
}

pub fn output(value: syn::ItemImpl, items: &BTreeMap<syn::Ident, Item>, lib: &syn::Path) -> TokenStream {
    let self_ty = *value.self_ty;
    let (impl_generics, ty_generics, where_clause) = value.generics.split_for_impl();

    let impl_input = ImplInput::get(value.items);

    let len = match self_ty.clone() {
        syn::Type::Path(value) => value.path.require_ident().ok().and_then(|ident| {
            items.get(ident).map(|item| item_len(item, lib))
        }),
        _ => None,
    }.or_else(|| impl_input.len).unwrap();

    let buf_ty = impl_input.buf.ty;
    let (buf_impl_generics, buf_ty_generics, _) = impl_input.buf.generics.split_for_impl();

    quote! {
        impl #impl_generics #lib::Entry for #self_ty #ty_generics #where_clause {
            const LEN: usize = #len;
            type Buf #buf_impl_generics = #buf_ty #buf_ty_generics;

            fn buf<BV: #lib::entry::bytes::Variant>(bytes: #lib::entry::Bytes<BV, { #len }>) -> Self::Buf<BV> {
                #buf_ty(bytes, std::marker::PhantomData)
            }
            fn buf_mut_as_const<'a>(buf: &'a #lib::entry::BufMut<'a, Self>) -> #lib::entry::BufConst<'a, Self> {
                Self::buf(buf.0.as_const())
            }
            fn buf_owned_as_const(buf: &#lib::entry::BufOwned<Self>) -> #lib::entry::BufConst<'_, Self> {
                Self::buf(buf.0.as_const())
            }
            fn buf_owned_as_mut(buf: &mut #lib::entry::BufOwned<Self>) -> #lib::entry::BufMut<'_, Self> {
                Self::buf(buf.0.as_mut())
            }
            fn buf_rb_const<'a>(buf: &'a #lib::entry::BufConst<'a, Self>) -> #lib::entry::BufConst<'a, Self> {
                Self::buf(buf.0.rb_const())
            }
            fn buf_rb_mut<'a>(buf: &'a mut #lib::entry::BufMut<'a, Self>) -> #lib::entry::BufMut<'a, Self> {
                Self::buf(buf.0.rb_mut())
            }
            unsafe fn buf_detach<'b, BV: #lib::entry::bytes::variant::Ref>(buf: Self::Buf<BV>) -> Self::Buf<BV::Ref<'b>> {
                Self::buf(buf.0.detach())
            }
            fn buf_copy_to(src: #lib::entry::BufConst<'_, Self>, mut dst: #lib::entry::BufMut<'_, Self>) {
                dst.0.copy_from(&src.0)
            }
            fn buf_swap(mut a: #lib::entry::BufMut<'_, Self>, mut b: #lib::entry::BufMut<'_, Self>) {
                a.0.swap(&mut b.0)
            }
        }
    }
}