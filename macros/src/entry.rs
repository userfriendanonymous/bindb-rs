use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned};
pub use input::{Item, Value as Input};

// pub mod r#struct;
pub mod input;
// pub mod r#for;
pub mod expand;

pub fn derive(input: Input, lib: &syn::Path) -> TokenStream {
    expand::output(input, lib)
}

// entry! {
//     #[buf(IdkBuf)]
//     struct Idk<T: Smth> {
//         opt: Option<T>
//     }

//     impl<T: Smth> I for Idk<T> {
//         type Buf<BV> = IdkBuf<BV, T>;
//     }

//     impl<T: Smth> Codable for Idk<T> {}

//     buf! { OptionBuf<BV: bytes::Variant, T>(BV) }
//     impl<T> I for Option<T> {
//         const LEN: usize = T::LEN + 1;
//         type Buf<BV> = OptionBuf<BV, T>;
//     }
// }