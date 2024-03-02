use proc_macro::TokenStream;
use quote::quote;


pub fn derive(item: syn::ItemStruct, lenser_info: syn::ItemType, lib: syn::Path) -> TokenStream {
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
                        <#ty as #lib::Codable>::SIZE
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
                        #lib::Codable::encode(&self.#ident, &mut bytes.index_to(cursor));
                        cursor += <#ty as #lib::Codable>::SIZE;
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
                            let v = #lib::Codable::decode(&bytes.index_to(cursor));
                            cursor += <#ty as #lib::Codable>::SIZE;
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
                        pub fn #ident(&self) -> #lib::Lens<#item_ident, #ty> {
                            const CURSOR: usize = #cursor;
                            self.0.spawn(CURSOR)
                        }
                    });
                    cursor = quote! {
                        #cursor + <#ty as #lib::Codable>::SIZE
                    };
                }
            }
        },
        syn::Fields::Unnamed(fields) => {
            size = {
                let iter = fields.unnamed.clone().into_iter().map(|field| {
                    let ty = field.ty;
                    quote! {
                        <#ty as #lib::Codable>::SIZE
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
                        #lib::Codable::encode(&self.#idx, &mut bytes.index_to(cursor));
                        cursor += <#ty as #lib::Codable>::SIZE;
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
                            let v = #lib::Codable::decode(&bytes.index_to(cursor));
                            cursor += <#ty as #lib::Codable>::SIZE;
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

    quote! {
        #item

        #lenser_vis struct #lenser_ident(#lib::lenser::Root<#item_ident>);

        impl #lenser_ident {
            #( #lenser_impl )*
        }

        impl #lib::Codable for #item_ident {
            const SIZE: usize = #size;

            fn encode(&self, bytes: &mut #lib::buf::bytes::Mut<'_, Self>) {
                #encode_fn
            }

            fn decode(bytes: &#lib::buf::bytes::Ref<'_, Self>) -> Self {
                #decode_fn
            }

            type Lenser = #lenser_ident;
            fn lenser_from_root(root: #lib::lenser::Root<Self>) -> Self::Lenser {
                #lenser_ident(root)
            }
        }
    }.into()
}