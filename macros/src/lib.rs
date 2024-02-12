use proc_macro::TokenStream;
use syn::{parse::{Parse, ParseStream}, punctuated::Punctuated};

mod codable;

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

#[proc_macro_derive(Codable)]
pub fn derive_codable(stream: TokenStream) -> TokenStream {
    // codable::derive(stream)
    stream
}