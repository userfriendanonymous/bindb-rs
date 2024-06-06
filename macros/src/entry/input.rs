use std::collections::BTreeMap;

use syn::{parse::Parse, spanned::Spanned};

pub struct Value {
    pub items: BTreeMap<String, Item>,
    pub impls: Vec<syn::ItemImpl>,
    pub macros: Vec<syn::ItemMacro>,
}

impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut impls = Vec::new();
        let mut macros = Vec::new();
        let mut items = BTreeMap::new();

        while !input.is_empty() {
            match input.parse()? {
                syn::Item::Struct(value) => {
                    // panic!("{}", value.ident.clone().to_string());
                    items.insert(value.ident.to_string(), Item::Struct(value));
                }
                syn::Item::Enum(value) => {
                    items.insert(value.ident.to_string(), Item::Enum(value));
                }
                syn::Item::Impl(value) => {
                    impls.push(value);
                }
                syn::Item::Macro(value) => {
                    macros.push(value);
                }
                value => return Err(syn::Error::new(value.span(), "Unexpected item.")),
            }
        }
        Ok(Self {
            items,
            impls,
            macros,
        })

        // let mut set_item = |span: proc_macro2::Span, value: Item| {
        //     if item.is_none() {
        //         item = Some(value);
        //         Ok(())
        //     } else {
        //         Err(syn::Error::new(span, "Only single item is allowed."))
        //     }
        // };

        // while !input.is_empty() {
        //     match input.parse()? {
        //         syn::Item::Enum(value) => {
        //             set_item(value.span(), Item::Enum(value))?;
        //         }
        //         syn::Item::Struct(value) => {
        //             set_item(value.span(), Item::Struct(value))?;
        //         }
        //         syn::Item::Type(value) => match value.ident.to_string().as_str() {
        //             "Buf" => {
        //                 buf = Some(value)
        //             },

        //             "For" => {
        //                 set_item(value.span(), Item::For(value))?;
        //             },
        //             _ => {
        //                 return Err(syn::Error::new(
        //                     value.ident.span(),
        //                     "Unexpected type alias. Only `Buf` is allowed.",
        //                 ))
        //             }
        //         },
        //         syn::Item::Impl(value) => {

        //     let generics = value.generics;
        // let target = *value.self_ty;
        //     let kind = match
        //         value.trait_.ok_or(syn::Error::new(value.span(), "Must have trait"))?
        //             .1.require_ident().map_err(|x| x)?.to_string().as_str()
        //     {
        //         "Instance" => {
        //             let mut len = None;
        //             let mut buf = None;
        //             for item in value.items {
        //                 match item {
        //                     syn::ImplItem::Const(value) => match value.ident.to_string().as_str() {
        //                         "LEN" => len = Some(value),
        //                     }
        //                     syn::ImplItem::Type(value) => match value.ident.to_string().as_str() {
        //                         "Buf" => buf = Some(value)
        //                     },
        //                     _ => return Err(syn::Error::new(item.span(), "Unexpected item.")),
        //                 }
        //             }
        //             ImplKind::Instance { buf: buf.ok_or(syn::Error::new(value.span(), "Buf expected"))?, len }
        //         }
        //     };
        //     impls.push(Impl {
        //         generics,
        //         target,
        //         kind
        //     });
        // },
        // other => return Err(syn::Error::new(other.span(), "Unexpected item.")),
        // }
        // }

        // let Some(item) = item else {
        //     return Err(input.error("Item not found."));
        // };

        // Ok(Self { item, meta: Meta { buf: buf.ok_or(syn::Error::new(input.span(), "Buf expected."))?, impls } })
    }
}

#[derive(Clone)]
pub enum Item {
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct),
}
