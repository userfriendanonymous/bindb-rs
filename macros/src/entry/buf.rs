
use quote::quote;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use to_phantom::ToPhantom;

pub struct Input {
    vis: syn::Visibility,
    ident: syn::Ident,
    generics: syn::Generics,
    entry_ty: syn::Type,
    ptr_ident: syn::Ident,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item = input.parse::<syn::ItemStruct>()?;
        if let syn::Fields::Unnamed(fields) = item.fields {
            let fields_span = fields.span();
            let mut fields_iter = fields.unnamed.into_iter();
            let field0 = fields_iter.next().ok_or(syn::Error::new(fields_span, "First field expected"))?;
            let field1 = fields_iter.next().ok_or(syn::Error::new(fields_span, "Second field expected"))?;

            let entry_ty = field0.ty;

            let ptr_ident = if let syn::Type::Path(path) = field1.ty {
                path.path.require_ident()?.clone()
            } else {
                Err(syn::Error::new(field1.ty.span(), "Must be an ident"))?
            };

            Ok(Self {
                vis: item.vis,
                ident: item.ident,
                generics: item.generics,
                entry_ty,
                ptr_ident
            })
        } else {
            Err(syn::Error::new(item.fields.span(), "Unnamed fields expected"))
        }
    }
}

pub fn output(mut input: Input, lib: &syn::Path) -> TokenStream {
    let ptr_trait_bound: syn::TraitBound = syn::parse2(quote! { #lib::entry::Ptr }).unwrap();

    let vis = input.vis;
    let ident = input.ident;
    let ptr_ident = input.ptr_ident;
    let entry_ty = input.entry_ty;

    input.generics.type_params_mut().find(|p| p.ident == ptr_ident).unwrap()
        .bounds.push(syn::TypeParamBound::Trait(ptr_trait_bound));

    let (generics_params, ty_generics, where_clause) = input.generics.split_for_impl();
    let phantom = input.generics.to_phantom();
    
    quote! {
        #vis struct #ident #generics_params (
            #ptr_ident,
            #phantom
        ) #where_clause;

        impl #generics_params Clone for #ident #ty_generics {
            fn clone(&self) -> Self {
                Self(self.0, self.1)
            }
        }

        impl #generics_params Copy for #ident #ty_generics {}

        impl #generics_params #lib::entry::Readable<#entry_ty> for #ident #ty_generics {
            fn write_to(self, buf: #lib::entry::BufMut<#entry_ty>) {
                #lib::entry::buf_copy_to::<#entry_ty>(#ident(self.0.to_const(), ::std::marker::PhantomData), buf);
            }
        }
    }
}
