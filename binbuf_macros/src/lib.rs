
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input};

mod fixed;
mod dynamic;

struct InputWithLibPath<Rest> {
    path: syn::Path,
    rest: Rest,
}

impl<Rest: Parse> Parse for InputWithLibPath<Rest> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = syn::Path::parse(input)?;
        input.parse::<syn::Token![;]>()?;
        Ok(Self {
            path,
            rest: input.parse()?
        })
    }
}
 
#[proc_macro]
pub fn derive_fixed(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as InputWithLibPath<fixed::Input>);
    fixed::derive(item.rest, &item.path).into()
}

#[proc_macro]
pub fn derive_fixed_buf(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as InputWithLibPath<fixed::buf::Input>);
    fixed::buf::output(item.rest, &item.path).into()
}

#[proc_macro]
pub fn derive_dynamic(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as InputWithLibPath<dynamic::Input>);
    dynamic::derive(item.rest, &item.path).into()
}

#[proc_macro]
pub fn derive_dynamic_buf(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as InputWithLibPath<dynamic::buf::Input>);
    dynamic::buf::output(item.rest, &item.path).into()
}

struct MacroWithCratePath {
    input_path: syn::Path,
    output_name: syn::Ident,
}

impl Parse for MacroWithCratePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input_path = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let output_name = input.parse()?;
        Ok(Self {
            input_path,
            output_name
        })
    }
}

#[proc_macro]
pub fn macro_with_crate_path(input: TokenStream) -> TokenStream {
    let MacroWithCratePath { input_path, output_name } = parse_macro_input!(input as MacroWithCratePath);
    quote! {
        #[macro_export]
        macro_rules! #output_name {
            ($($arg:tt)*) => {
                $crate::#input_path! {$crate;$($arg)*}
            }
        }
    }.into()
}
