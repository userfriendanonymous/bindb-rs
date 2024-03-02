use proc_macro::TokenStream;
use quote::quote;

pub fn derive(item: syn::ItemEnum, lib: syn::Path) -> TokenStream {
    let item_ident = item.ident.clone();

    let (tag_size, tag_ty) = match item.variants.len() {
        x if x < 256 => (1, quote! { ::std::primitive::u8 }),
        _ => panic!("Not yet supported")
    };

    let encode_fn = {
        let match_branches = item.variants.iter().enumerate().map(|(variant_id, variant)| {
            let variant_ident = &variant.ident;
            match variant.fields {
                syn::Fields::Unit => {
                    quote! {
                        Self::#variant_ident => {
                            <#tag_ty as #lib::Codable>::encode(#variant_id, &mut bytes.index_to(0));
                            bytes[<#tag_ty as #lib::Codable>::SIZE .. <Self as #lib::Codable>::SIZE].fill(0);
                        }
                    }
                },
                syn::Fields::Named(fields) => {
                    let match_fields = fields.named.iter().map(|field| field.ident.as_ref().unwrap());
                    let encode_fields = fields.named.iter().map(|field| {
                        let field_ident = field.ident.as_ref().unwrap();
                        let field_ty = &field.ty;
                        quote! {
                            self.#field_ident.encode(&mut bytes.index_to(cursor));
                            cursor += <#field_ty as #lib::Codable>::SIZE;
                        }
                    });

                    quote! {
                        Self::#variant_ident { #( #match_fields ),* } => {
                            let mut cursor: usize = 0;
                            #( #encode_fields )*
                            bytes[cursor .. <Self as #lib::Codable>::SIZE].fill(0);
                        }
                    }
                },
                _ => panic!("Unnamed fields not yet supported")
            }
        });

        quote! {
            match self {
                #( #match_branches )*
            }
        }
    };
    

    quote! {
        #item

        impl #lib::Codable for #item_ident {
            const SIZE: usize = #size;

            fn encode(&self, bytes: &mut #lib::buf::bytes::Mut<'_, Self>) {
                #encode_fn
            }

            fn decode(bytes: &#lib::buf::bytes::Ref<'_, Self>) -> Self {
                #decode_fn
            }

            type Lenser = #lib::lenser::Empty;
            fn lenser_from_root(root: #lib::lenser::Root<Self>) -> Self::Lenser {
                #lib::lenser::Empty::new(root)
            }
        }
    }.into()
}