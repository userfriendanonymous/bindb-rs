use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, ItemStruct};
use super::InputWithLibPath;

pub struct StructDeriveInput {
    item: ItemStruct,
}

impl Parse for StructDeriveInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item = input.parse::<syn::ItemStruct>()?;
        Ok(Self {
            item,
        })
    }
}

impl StructDeriveInput {
    pub fn output(self, lib_path: syn::Path) -> TokenStream {
        let ident = self.item.ident;
        let fields = self.item.fields;

        let encode_function = {
            let fields_op = match fields {
                syn::Fields::Named(fields) => {
                    fields.named.into_iter().map(|field| {
                        let ident = field.ident.unwrap();
                        let ty = field.ty;
                        quote! {
                            let size = <#ty as #lib_path::Codable>::size();
                            #lib_path::Codable::encode(&self.#ident, bytes[cursor .. cursor + size]);
                            cursor += size;
                        }
                    })
                },
                _ => unimplemented!()
            };
            quote! {
                let mut cursor: usize = 0;
                #( #fields_op )*
            }
        };

        let output = quote! {
            impl #lib_path::Codable for #ident {
                fn encode(&self, bytes: &mut [u8]) {
                    #encode_function
                }
            }
        };
        output.into()
    }
}
