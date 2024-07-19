#![feature(generic_const_exprs)]

pub mod storage;
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
