use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned};
use input::{Item, Value as Input};

pub mod r#struct;
pub mod input;
pub mod r#for;

pub fn derive(input: Input, lib_path: syn::Path) -> TokenStream {
    match input.item {
        Item::Struct(item) => {
            let buf_info = input
                .buf
                .ok_or("Provide a buf: type Lenser = SomeLenser;")
                .unwrap();
            r#struct::derive(item, buf_info, lib_path)
        }
        Item::Enum(item) => {
            panic!("Enums aren't yet supported")
        },
        Item::For()
    }
}
