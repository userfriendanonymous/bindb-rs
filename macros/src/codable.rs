use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned};

pub mod r#struct;
// pub mod r#enum;

pub struct Input {
    lenser: Option<syn::ItemType>,
    item: Item,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut item = None;
        let mut lenser = None;

        let mut set_item = |span: proc_macro2::Span, value: Item| {
            if item.is_none() {
                item = Some(value);
                Ok(())
            } else {
                Err(syn::Error::new(span, "Only single item is allowed."))
            }
        };

        while !input.is_empty() {
            match input.parse()? {
                syn::Item::Enum(value) => {
                    set_item(value.span(), Item::Enum(value))?;
                }
                syn::Item::Struct(value) => {
                    set_item(value.span(), Item::Struct(value))?;
                }
                syn::Item::Type(value) => match value.ident.to_string().as_str() {
                    "Lenser" => {
                        if lenser.is_none() {
                            lenser = Some(value)
                        } else {
                            return Err(syn::Error::new(
                                value.ident.span(),
                                "`Lenser` is defined twice.",
                            ));
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            value.ident.span(),
                            "Unexpected type alias. Only `Lenser` is allowed.",
                        ))
                    }
                },
                other => return Err(syn::Error::new(other.span(), "Unexpected item.")),
            }
        }

        let Some(item) = item else {
            return Err(input.error("Item not found."));
        };

        Ok(Self { lenser, item })
    }
}

pub enum Item {
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct),
}

pub fn derive(input: Input, lib_path: syn::Path) -> TokenStream {
    match input.item {
        Item::Struct(item) => {
            let lenser_info = input
                .lenser
                .ok_or("Provide a lenser: type Lenser = SomeLenser;")
                .unwrap();
            r#struct::derive(item, lenser_info, lib_path)
        }
        Item::Enum(item) => {
            panic!("Enums aren't yet supported")
        }
    }
}
