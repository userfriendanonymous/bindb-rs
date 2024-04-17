use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;

// buf! { struct OptionBuf<BV, T>(BV, Option) where T: Clone; }

pub fn output(value: syn::ItemMacro, items: &BTreeMap<syn::Ident, super::super::Item>, lib: &syn::Path) -> TokenStream {
    match value.mac.path.require_ident().unwrap().to_string().as_str() {
        // "buf" => {
        //     items.get(key)
        //     let input = value.mac.parse_body::<BufInput>().unwrap();
        // },
        _ => panic!("No such macro expected.")
    }
}