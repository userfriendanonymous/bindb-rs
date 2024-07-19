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
    }
}

#[derive(Clone)]
pub enum Item {
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct),
}

impl Item {
    pub fn attrs(&self) -> &Vec<syn::Attribute> {
        match self {
            Item::Enum(item) => &item.attrs,
            Item::Struct(item) => &item.attrs,
        }
    }
}