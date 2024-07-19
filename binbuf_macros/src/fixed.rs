use proc_macro2::TokenStream;
pub use input::{Item, Value as Input};

pub mod input;
pub mod expand;
pub mod buf;

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