#![feature(generic_const_exprs)]
#![feature(associated_const_equality)]
#![feature(generic_const_items)]
#![feature(trait_alias)]

extern crate macros;

macro_with_crate_path! { macros::derive_entry; entry }
macro_with_crate_path! { macros::derive_entry_buf; entry_buf }
pub use entry::Instance as Entry;
use macros::macro_with_crate_path;
pub use lens::Instance as Lens;
pub mod collection;

pub mod entry;
pub mod lens;
mod private;
mod utils;

// fn benchmark<R>(msg: &'static str, f: impl FnOnce() -> R) -> R {
//     let start = std::time::Instant::now();
//     let r = f();
//     let elapsed = start.elapsed().as_millis();
//     println!("{}: {} ms.", msg, elapsed);
//     r
// }
