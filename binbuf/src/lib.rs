
#![feature(min_specialization)]
#![feature(generic_const_exprs)]

pub extern crate macros;
pub use macros as macros_reexp;

macro_with_crate_path! { macros::derive_fixed; fixed }
macro_with_crate_path! { macros::derive_fixed_buf; fixed_buf }
macro_with_crate_path! { macros::derive_dynamic; dynamic }
macro_with_crate_path! { macros::derive_dynamic_buf; dynamic_buf }

use macros::macro_with_crate_path;
pub use fixed::Instance as Fixed;
pub use dynamic::Instance as Dynamic;
pub use bytes_ptr::{Instance as BytesPtr, Const as BytesPtrConst, Mut as BytesPtrMut};
pub use entry::{Instance as Entry, Buf, BufConst, BufMut, buf_to_const};

pub mod entry;
pub mod bytes_ptr;
pub mod fixed;
pub mod dynamic;
pub mod impls;
mod private;
mod utils;
mod tests;

// fn benchmark<R>(msg: &'static str, f: impl FnOnce() -> R) -> R {
//     let start = std::time::Instant::now();
//     let r = f();
//     let elapsed = start.elapsed().as_millis();
//     println!("{}: {} ms.", msg, elapsed);
//     r
// }
