use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned};

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
                },
                syn::Item::Struct(value) => {
                    set_item(value.span(), Item::Struct(value))?;
                },
                syn::Item::Type(value) => {
                    match value.ident.to_string().as_str() {
                        "Lenser" => {
                            if lenser.is_none() {
                                lenser = Some(value)
                            } else {
                                return Err(syn::Error::new(value.ident.span(), "`Lenser` is defined twice."))
                            }
                        },
                        _ => return Err(syn::Error::new(value.ident.span(), "Unexpected type alias. Only `Lenser` is allowed."))
                    }
                },
                other => return Err(syn::Error::new(other.span(), "Unexpected item."))
            }
        }

        let Some(item) = item else {
            return Err(input.error("Item not found."))
        };

        Ok(Self {
            lenser,
            item,
        })
    }
}

pub enum Item {
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct)
}

pub fn derive(input: Input, lib_path: syn::Path) -> TokenStream {
    match input.item {
        Item::Struct(item) => {
            let lenser_info = input.lenser.ok_or("Provide a lenser: type Lenser = SomeLenser;").unwrap();
            derive_struct(item, lenser_info, lib_path)
        },
        Item::Enum(item) => {
            panic!("Enums aren't yet supported")
        }
    }
}

pub fn derive_struct(item: syn::ItemStruct, lenser_info: syn::ItemType, lib_path: syn::Path) -> TokenStream {
    let item_ident = item.ident.clone();

    let lenser_ident = match lenser_info.ty.as_ref() {
        syn::Type::Path(path) => path.path.require_ident().unwrap(),
        _ => panic!("Type must be an indent.")
    };
    let lenser_vis = lenser_info.vis.clone();

    let encode_fn;
    let decode_fn;
    let size;
    let mut lenser_impl = Vec::new();

    match item.fields.clone() {
        syn::Fields::Unit => {
            encode_fn = quote! {};
            decode_fn = quote! {
                Self
            };
            size = quote! { 0 };
        }

        syn::Fields::Named(fields) => {
            size = {
                let iter = fields.named.clone().into_iter().map(|field| {
                    let ty = field.ty;
                    quote! {
                        <#ty as #lib_path::Codable>::SIZE
                    }
                });
                quote! {
                    #( #iter )+*
                }
            };

            encode_fn = {
                let iter = fields.named.clone().into_iter().map(|field| {
                    let ident = field.ident.unwrap();
                    let ty = field.ty;
                    quote! {
                        #lib_path::Codable::encode(&self.#ident, &mut bytes.index_to(cursor));
                        cursor += <#ty as #lib_path::Codable>::SIZE;
                    }
                });
                quote! {
                    let mut cursor: usize = 0;
                    #( #iter )*
                }
            };

            decode_fn = {
                let iter = fields.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        #ident: {
                            let v = #lib_path::Codable::decode(&bytes.index_to(cursor));
                            cursor += <#ty as #lib_path::Codable>::SIZE;
                            v
                        }
                    }
                });
                quote! {
                    let mut cursor: usize = 0;
                    Self {
                        #( #iter ),*
                    }
                }
            };

            {
                let mut cursor = quote! { 0 };
                for field in fields.named.clone() {
                    let ident = field.ident.unwrap();
                    let ty = field.ty;
                    lenser_impl.push(quote! {
                        pub fn #ident(&self) -> #lib_path::Lens<#item_ident, #ty> {
                            const CURSOR: usize = #cursor;
                            self.0.spawn(CURSOR)
                        }
                    });
                    cursor = quote! {
                        #cursor + <#ty as #lib_path::Codable>::SIZE
                    };
                }
            }
        },
        syn::Fields::Unnamed(fields) => {
            size = {
                let iter = fields.unnamed.clone().into_iter().map(|field| {
                    let ty = field.ty;
                    quote! {
                        <#ty as #lib_path::Codable>::SIZE
                    }
                });
                quote! {
                    #( #iter )+*
                }
            };

            encode_fn = {
                let iter = fields.unnamed.clone().into_iter().enumerate().map(|(idx, field)| {
                    let ty = field.ty;
                    let idx = syn::Index::from(idx);
                    quote! {
                        #lib_path::Codable::encode(&self.#idx, &mut bytes.index_to(cursor));
                        cursor += <#ty as #lib_path::Codable>::SIZE;
                    }
                });

                quote! {
                    let mut cursor: usize = 0;
                    #( #iter )*
                }
            };

            decode_fn = {
                let iter = fields.unnamed.into_iter().enumerate().map(|(_idx, field)| {
                    let ty = field.ty;
                    quote! {
                        {
                            let v = #lib_path::Codable::decode(&bytes.index_to(cursor));
                            cursor += <#ty as #lib_path::Codable>::SIZE;
                            v
                        }
                    }
                });
                quote! {
                    let mut cursor: usize = 0;
                    Self (
                        #( #iter ),*
                    )
                }
            }
        }
    }

    let output = quote! {
        #item

        #lenser_vis struct #lenser_ident(#lib_path::lenser::Root<#item_ident>);

        impl #lenser_ident {
            #( #lenser_impl )*
        }

        impl #lib_path::Codable for #item_ident {
            const SIZE: usize = #size;

            fn encode(&self, bytes: &mut #lib_path::buf::bytes::Mut<'_, Self>) {
                #encode_fn
            }

            fn decode(bytes: &#lib_path::buf::bytes::Ref<'_, Self>) -> Self {
                #decode_fn
            }

            type Lenser = #lenser_ident;
            fn lenser_from_root(root: #lib_path::lenser::Root<Self>) -> Self::Lenser {
                #lenser_ident(root)
            }
        }
    };
    output.into()
}