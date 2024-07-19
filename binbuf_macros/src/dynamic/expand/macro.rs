use std::collections::BTreeMap;
use proc_macro2::TokenStream;

// buf! { struct OptionBuf<BV, T>(BV, Option) where T: Clone; }

pub fn output(value: syn::ItemMacro, _items: &BTreeMap<String, super::super::Item>, lib: &syn::Path) -> TokenStream {
    match value.mac.path.require_ident().unwrap().to_string().as_str() {
        "buf" => {
            super::super::buf::output(value.mac.parse_body().unwrap(), lib)
        },
        _ => panic!("No such macro expected.")
    }
}
