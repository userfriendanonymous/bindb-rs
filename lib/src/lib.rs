#![feature(generic_const_exprs)]
use std::{fs::File, io::{Seek, Write}, marker::PhantomData, path::Path, sync::RwLock};

use fmmap::{MmapFileExt as _, MmapFileMut, MmapFileMutExt};
use memmap::Mmap;
pub use lens::{Value as Lens, ToValue as ToLens};
pub use codable::Instance as Codable;
pub use lensable::Instance as Lensable;
pub use collection::Value as Collection;
pub use buf::{Instance as Buf, AsInstance as AsBuf};

pub mod codable;
pub mod entry;
pub mod lens;
pub mod collection;
pub mod buf;
pub mod lensable;
mod private;
mod utils;

fn benchmark<R>(msg: &'static str, f: impl FnOnce() -> R) -> R {
    let start = std::time::Instant::now();
    let r = f();
    let elapsed = start.elapsed().as_millis();
    println!("{}: {} ms.", msg, elapsed);
    r
}



// #[test]
// fn benchmarks() {
//     // let mut file = File::create("./local/len_test").unwrap();
//     // let len = 1321528398;
//     // file.set_len(8 + Item::size() as u64 * len).unwrap();
//     // let mut file_map = MmapFileMut::open("./local/len_test").unwrap();
//     // file_map.write_u64(len, 0).unwrap();
//     let mut db = benchmark("Open", || Collection::<Item>::open("./local/len_test").unwrap());

//     let total = 1321528398;

//     benchmark("Some ops", || {
//         db.set(Lens::to_self(), entry::Id::from_u64(0), &Item {
//             bar: 1,
//             foo: Some(1.11)
//         }).unwrap();

//         db.set(Lens::to_self(), entry::Id::from_u64(total - 2), &Item {
//             bar: 999,
//             foo: Some(999.999)
//         }).unwrap();

//         db.copy(Lens::to_self(), entry::Id::from_u64(total - 2), entry::Id::from_u64(1)).unwrap();

//         let buf1 = db.buf_ref(Lens::to_self(), entry::Id::from_u64(0)).unwrap();
//         let buf2 = db.buf_ref(Lens::to_self(), entry::Id::from_u64(total - 2)).unwrap();
//         println!("#0: {:?}", buf1.decode());
//         println!("#(total - 2): {:?}", buf2.decode());
//         println!("#1: {:?}", db.get(Lens::to_self(), entry::Id::from_u64(1)));
//     });

//     // db.add(&Item {
//     //     foo: None,
//     //     bar: 20,
//     // }).unwrap();

//     // benchmark("insert", || {
//     //     for i in 0 .. total {
//     //         let id = db.add(&Item {
//     //             foo: Some(i as f32 + 0.53),
//     //             bar: i
//     //         }).unwrap();
//     //     }
//     // });
//     // db.set(Id(5), Item::foo(), &None).unwrap();

//     // benchmark("read", || {
//     //     for i in 0 .. total {
//     //         db.get(Lens::to_self(), entry::Id::from(i)).unwrap();
//     //     }
//     // });

//     // let item = benchmark("find_exact", || {
//     //     db.find_exact(Item::bar(), &(total - 10)).unwrap()
//     // });

//     // let item = benchmark("find", || {
//     //     db.find(Item::bar(), |&x| x >= total - 10).unwrap()
//     // });

//     // let item = benchmark("find_full", || {
//     //     db.find(Lens::to_self(), |x| x.bar >= total - 10).unwrap()
//     // });

//     // benchmark("set", || {
//     //     for i in 0 .. total {
//     //         db.set(Lens::to_self(), entry::Id::from(i), &Item {
//     //             foo: None,
//     //             bar: 13490345
//     //         }).unwrap();
//     //     }
//     // });

//     let bar = db.get(Lens::to_self(), entry::Id::from(0)).unwrap();
//     println!("{bar:?}");
    
// }

// struct ItemLens(usize);

// impl ItemLens {
//     pub const FOO: Self = Self(0);
//     pub const BAR: Self = Self(<Option::<f32> as Codable>::SIZE);
// }

// #[derive(Debug)]
// struct Item {
//     pub foo: Option<f32>,
//     pub bar: u64,
// }

// impl Codable for Item {
//     const SIZE: usize = <Option::<f32> as Codable>::SIZE + <u64 as Codable>::SIZE;
//     fn encode(&self, bytes: &mut [u8]) {
//         let foo_size = <Option::<f32> as Codable>::SIZE;
//         self.foo.encode(&mut bytes[0 .. foo_size as usize]);
//         self.bar.encode(&mut bytes[foo_size as usize ..]);
//     }

//     fn decode(bytes: &[u8]) -> Self where Self: Sized {
//         let foo_size = <Option::<f32> as Codable>::SIZE;
//         Self {
//             foo: Codable::decode(&bytes[0 .. foo_size]).map_err(|_| "foo".to_string())?,
//             bar: Codable::decode(&bytes[foo_size ..]).map_err(|_| "bar".to_string())?
//         }
//     }
// }
