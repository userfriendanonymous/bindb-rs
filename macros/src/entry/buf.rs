
use quote::quote;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use to_phantom::ToPhantom;

pub struct Input {
    vis: syn::Visibility,
    ident: syn::Ident,
    generics: syn::Generics,
    entry_ty: syn::Type,
    bv_ident: syn::Ident,
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

            let bv_ident = if let syn::Type::Path(path) = field1.ty {
                path.path.require_ident()?.clone()
            } else {
                Err(syn::Error::new(field1.ty.span(), "Must be an ident"))?
            };

            Ok(Self {
                vis: item.vis,
                ident: item.ident,
                generics: item.generics,
                entry_ty,
                bv_ident
            })
        } else {
            Err(syn::Error::new(item.fields.span(), "Unnamed fields expected"))
        }
    }
}


// buf! { struct OptionBuf<BV, T>(Option, BV) where T: Clone; }

pub fn output(mut input: Input, lib: &syn::Path) -> TokenStream {
    let bv_trait_bound: syn::TraitBound = syn::parse2(quote! { #lib::entry::bytes::Variant }).unwrap();

    let vis = input.vis;
    let ident = input.ident;
    let bv_ident = input.bv_ident;
    let entry_ty = input.entry_ty;

    input.generics.type_params_mut().find(|p| p.ident == bv_ident).unwrap()
        .bounds.push(syn::TypeParamBound::Trait(bv_trait_bound));

    let (generics_params, _, where_clause) = input.generics.split_for_impl();
    let phantom = input.generics.to_phantom();
    
    quote! {
        #vis struct #ident #generics_params (
            #lib::entry::Bytes<#bv_ident>,
            #phantom
        ) #where_clause;
    }
}
