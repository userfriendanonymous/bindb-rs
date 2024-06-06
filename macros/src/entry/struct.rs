use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;


pub fn derive(item: syn::ItemStruct, meta: super::input::Meta, lib: syn::Path) -> TokenStream {
    let item_ident = item.ident.clone();

    let buf_info = meta.buf;


    let buf_ident = match buf_info.ty.as_ref() {
        syn::Type::Path(path) => path.path.require_ident().expect("Idk hmm"),
        _ => panic!("Type must be an indent.")
    };
    panic!("{}", buf_ident.to_string());
    let buf_vis = buf_info.vis.clone();

    let codable_encode;
    let codable_decode;
    let len;
    let mut buf_impl = Vec::new();

    match item.fields.clone() {
        syn::Fields::Unit => {
            codable_encode = quote! {};
            codable_decode = quote! {
                Self
            };
            len = quote! { 0 };
        }

        syn::Fields::Named(fields) => {
            codable_encode = {
                let iter = fields.named.clone().into_iter().map(|field| {
                    let ident = field.ident.unwrap();
                    let ty = field.ty;
                    quote! {
                        <#ty as #lib::entry::Codable>::encode(&self.#ident, <#ty as #lib::Entry>::buf(unsafe { buf.0.const_index(cursor) }));
                        cursor += <#ty as #lib::Entry>::LEN;
                    }
                });
                quote! {
                    let mut cursor: usize = 0;
                    #( #iter )*
                }
            };

            codable_decode = {
                let iter = fields.named.iter().map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        #ident: {
                            let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { buf.0.const_index(cursor) }));
                            cursor += <#ty as #lib::Entry>::LEN;
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
                    let f_ident = syn::Ident::new(&format!("field_{ident}"), ident.span());
                    buf_impl.push(quote! {
                        pub fn #f_ident(self) -> #buf_ident<BV> {
                            const START_INDEX: usize = #cursor;
                            #buf_ident(unsafe { self.0.const_index(START_INDEX) })
                        }
                    });
                    cursor = quote! {
                        #cursor + <#ty as #lib::Entry>::LEN
                    };
                }
                len = cursor;
            }
        },
        syn::Fields::Unnamed(fields) => {
            codable_encode = {
                let iter = fields.unnamed.iter().enumerate().map(|(idx, field)| {
                    let ty = &field.ty;
                    let idx = syn::Index::from(idx);
                    quote! {
                        <#ty as #lib::entry::Codable>::encode(&self.#idx, <#ty as #lib::Entry>::buf(unsafe { buf.0.const_index(cursor) }));
                        cursor += <#ty as #lib::Entry>::LEN;
                    }
                });

                quote! {
                    let mut cursor: usize = 0;
                    #( #iter )*
                }
            };

            codable_decode = {
                let iter = fields.unnamed.iter().enumerate().map(|(_idx, field)| {
                    let ty = &field.ty;
                    quote! {
                        {
                            let v = <#ty as #lib::entry::Codable>::decode(<#ty as #lib::Entry>::buf(unsafe { buf.0.const_index(cursor) }));
                            cursor += <#ty as #lib::Entry>::LEN;
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
            };

            {
                let mut cursor = quote! { 0 };
                for (idx, field) in fields.unnamed.clone().into_iter().enumerate() {
                    let ty = &field.ty;
                    let f_ident = syn::Ident::new(&format!("field_{idx}"), field.span());
                    buf_impl.push(quote! {
                        pub fn #f_ident(self) -> #buf_ident<BV> {
                            const START_INDEX: usize = #cursor;
                            #buf_ident(unsafe { self.0.const_index(START_INDEX) })
                        }
                    });
                    cursor = quote! {
                        #cursor + <#ty as #lib::Entry>::LEN
                    };
                }
                len = cursor;
            }
        }
    }

    quote! {
        #item

        #buf_vis struct #buf_ident<BV: #lib::entry::bytes::Variant>(#lib::entry::Bytes<BV, { #len }>);

        impl<BV: #lib::entry::bytes::Variant> #buf_ident<BV> {
            #( #buf_impl )*
        }

        impl #lib::Entry for #item_ident {
            const LEN: usize = #len;
            type Buf<BV: #lib::entry::bytes::Variant> = #buf_ident<BV>;

            fn buf<BV: #lib::entry::bytes::Variant>(bytes: #lib::entry::Bytes<BV, { #len }>) -> Self::Buf<BV> {
                #buf_ident(bytes)
            }
            fn buf_mut_as_const<'a>(buf: &'a #lib::entry::BufMut<'a, Self>) -> #lib::entry::BufConst<'a, Self> {
                #buf_ident(buf.0.as_const())
            }
            fn buf_owned_as_const(buf: &#lib::entry::BufOwned<Self>) -> #lib::entry::BufConst<'_, Self> {
                #buf_ident(buf.0.as_const())
            }
            fn buf_owned_as_mut(buf: &mut #lib::entry::BufOwned<Self>) -> #lib::entry::BufMut<'_, Self> {
                #buf_ident(buf.0.as_mut())
            }
            fn buf_rb_const<'a>(buf: &'a #lib::entry::BufConst<'a, Self>) -> #lib::entry::BufConst<'a, Self> {
                #buf_ident(buf.0.rb_const())
            }
            fn buf_rb_mut<'a>(buf: &'a mut #lib::entry::BufMut<'a, Self>) -> #lib::entry::BufMut<'a, Self> {
                #buf_ident(buf.0.rb_mut())
            }
            unsafe fn buf_detach<'b, BV: #lib::entry::bytes::variant::Ref>(buf: Self::Buf<BV>) -> Self::Buf<BV::Ref<'b>> {
                #buf_ident(buf.0.detach())
            }
            fn buf_copy_to(src: #lib::entry::BufConst<'_, Self>, mut dst: #lib::entry::BufMut<'_, Self>) {
                dst.0.copy_from(&src.0)
            }
            fn buf_swap(mut a: #lib::entry::BufMut<'_, Self>, mut b: #lib::entry::BufMut<'_, Self>) {
                a.0.swap(&mut b.0)
            }
        }

        impl #lib::entry::Codable for #item_ident {
            fn encode(&self, buf: #lib::entry::BufMut<'_, Self>) {
                #codable_encode
            }
            fn decode(buf: #lib::entry::BufConst<'_, Self>) -> Self {
                #codable_decode
            }
        }
    }.into()
}

// entry! {
//     type Buf = CoolBuf;

//     #[derive(Clone, Debug)]
//     struct Cool {
//         field: u64,
//         other: Option<f32>,
//     }
// }