use syn::{parse::Parse, spanned::Spanned};

pub struct Meta {
    pub buf: Option<syn::ItemType>,
    pub len: Option<syn::ItemConst>,
}

pub struct Value {
    item: Item,
    meta: Meta,
}

impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut item = None;
        let mut meta = Meta {
            buf: None,
            len: None,
        };

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
                    "Buf" => {
                        if meta.buf.is_none() {
                            meta.buf = Some(value)
                        } else {
                            return Err(syn::Error::new(
                                value.ident.span(),
                                "`Buf` is defined twice.",
                            ));
                        }
                    },

                    "For" => {
                        set_item(value.span(), Item::For(*value.ty))?;
                    },
                    _ => {
                        return Err(syn::Error::new(
                            value.ident.span(),
                            "Unexpected type alias. Only `Buf` is allowed.",
                        ))
                    }
                },
                syn::Item::Const(value) => match value.ident.to_string().as_str() {
                    "LEN" => {
                        if meta.len.is_none() {
                            meta.len = Some(value)
                        } else {
                            return Err(syn::Error::new(
                                value.ident.span(),
                                "`LEN` is defined twice.",
                            ));
                        }
                    }
                }
                other => return Err(syn::Error::new(other.span(), "Unexpected item.")),
            }
        }

        let Some(item) = item else {
            return Err(input.error("Item not found."));
        };

        Ok(Self { item, meta })
    }
}

pub enum Item {
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct),
    For(syn::Type),
}