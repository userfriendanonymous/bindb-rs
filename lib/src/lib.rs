#![feature(generic_const_exprs)]
#![feature(associated_const_equality)]
#![feature(generic_const_items)]
#![feature(trait_alias)]

extern crate macros;

macro_with_crate_path! { macros::derive_entry; entry }
macro_with_crate_path! { macros::derive_entry_buf; entry_buf }
// pub use lens::{Value as Lens, producer as lenser};
// pub use codable::Instance as Codable;
// pub use collection::Value as Collection;
// pub use buf::{Instance as Buf, AsInstance as AsBuf};
// pub use codable::Instance as Codable;
// pub use lens::Instance as Lens;
// pub use buf::Instance as Buf;
pub use entry::Instance as Entry;
use macros::macro_with_crate_path;
// pub use ownership::Instance as Ownership;
pub use lens::Instance as Lens;
// pub mod buf;
pub mod collection;

pub mod entry;
pub mod lens;
// pub mod ref_variant;
// pub mod ownership;
mod private;
mod utils;

// fn benchmark<R>(msg: &'static str, f: impl FnOnce() -> R) -> R {
//     let start = std::time::Instant::now();
//     let r = f();
//     let elapsed = start.elapsed().as_millis();
//     println!("{}: {} ms.", msg, elapsed);
//     r
// }

// #[test]
// fn benchmarks_million() {
//     let mut db = benchmark("Open", || {
//         Collection::<Item>::open("./local/test_db").unwrap()
//     });

//     let total = 1000000;

//     // let v = db.get(Item::lenser().foo(), 1.into()).unwrap();
//     // println!("{v:?}");

//     // benchmark("insert", || {
//     //     for idx in 0 .. total {
//     //         db.add(&Item {
//     //             foo: Some(idx as u32),
//     //             bar: idx,
//     //             ..Default::default()
//     //         }).unwrap();
//     //     }
//     // });

//     // for idx in 0 .. total {
//     //     let mut v = db.buf_ref(Lens::FULL, idx.into()).unwrap().to(Item::lenser().inner()).decode();
//     //     v.h2 += 1;
//     // }

//     // let inner_lens = Item::lenser().inner();

//     // benchmark("read_bg", || {
//     //     for idx in 0 .. total {
//     //         let mut v = db.buf_ref(Lens::FULL, idx.into()).unwrap().to(Item::lenser().inner()).decode();
//     //         v.h2 += 1;
//     //     }
//     // });

//     // benchmark("read_sm", || {
//     //     for idx in 0 .. total {
//     //         let mut v = db.buf_ref(inner_lens, idx.into()).unwrap().decode();
//     //         v.h2 += 1;
//     //     }
//     // });

//     // sm: 225575875, 225240375
//     // bg: 242453208, 242666500

//     benchmark("read", || {
//         for idx in 0..total {
//             db.get(Lens::FULL, entry::Id::from(idx)).unwrap();
//         }
//     });

//     benchmark("update", || {
//         for idx in 0..total {
//             db.set(
//                 Lens::FULL,
//                 entry::Id::from(idx),
//                 &Item {
//                     foo: Some(idx as u32),
//                     bar: idx,
//                     ..Default::default()
//                 },
//             )
//             .unwrap();
//         }
//     });
// }

// // #[test]
// fn benchmarks_large() {
//     // let mut file = File::create("./local/len_test").unwrap();
//     // let len = 1321528398;
//     // file.set_len(8 + Item::SIZE as u64 * len).unwrap();
//     // let mut file_map = MmapFileMut::open("./local/len_test").unwrap();
//     // file_map.write_u64(len, 0).unwrap();
//     let mut db = benchmark("Open", || {
//         Collection::<Item>::open("./local/len_test").unwrap()
//     });

//     let total = 1321528398;

//     benchmark("Some ops", || {
//         db.set(
//             Lens::FULL,
//             entry::Id::from_u64(0),
//             &Item {
//                 bar: 1,
//                 foo: Some(1),
//                 ..Default::default()
//             },
//         )
//         .unwrap();

//         db.set(
//             Lens::FULL,
//             entry::Id::from_u64(total - 2),
//             &Item {
//                 bar: 999,
//                 foo: Some(999),
//                 ..Default::default()
//             },
//         )
//         .unwrap();

//         db.copy(
//             Lens::FULL,
//             entry::Id::from_u64(total - 2),
//             entry::Id::from_u64(1),
//         )
//         .unwrap();

//         let buf1 = db.buf_ref(Lens::FULL, entry::Id::from_u64(0)).unwrap();
//         let buf2 = db
//             .buf_ref(Lens::FULL, entry::Id::from_u64(total - 2))
//             .unwrap();
//         println!("#0: {:?}", buf1.decode());
//         println!("#(total - 2): {:?}", buf2.decode());
//         println!("#1: {:?}", db.get(Lens::FULL, entry::Id::from_u64(1)));
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

//     let items_count = 1_000_000;

//     benchmark("set", || {
//         for i in 0..items_count {
//             db.set(Item::lenser().bar(), entry::Id::from_u64(i), &i)
//                 .unwrap();
//         }
//     });

//     benchmark("read", || {
//         for i in 0..items_count {
//             db.get(Lens::FULL, entry::Id::from(i)).unwrap();
//         }
//     });

//     let item = benchmark("find", || {
//         db.find(Item::lenser().bar(), |&x| x >= items_count)
//             .unwrap()
//     });

//     let item = benchmark("find_exact", || {
//         db.find_exact(Item::lenser().bar(), &items_count).unwrap()
//     });

//     // let item = benchmark("find_full", || {
//     //     db.find(Lens::FULL, |x| x.bar >= total - 10).unwrap()
//     // });

//     // benchmark("set", || {
//     //     for i in 0 .. total {
//     //         db.set(Lens::FULL, entry::Id::from(i), &Item {
//     //             foo: None,
//     //             bar: 13490345
//     //         }).unwrap();
//     //     }
//     // });
// }

// // struct ItemLenser(lenser::Root<Item>);

// // impl ItemLenser {
// //     fn spawn<T>(offset: usize) -> Lens<Item, T> {
// //         Item::lenser().0.spawn(offset)
// //     }

// //     fn foo(&self) -> Lens<Item, Option<u32>> {
// //         self.0.spawn(0)
// //     }

// //     fn bar(&self) -> Lens<Item, u64> {
// //         self.0.spawn(<Option<u32>>::SIZE)
// //     }
// // }


// codable! {
//     type Lenser = ItemLenser;
//     #[derive(Debug, Default)]
//     struct Item {
//         pub inner: ItemInner,
//         pub foo: Option<u32>,
//         pub bar: u64,
//         pub wow: u64,
//         pub h1: u64,
//         pub h2: u64,
//         pub h3: u64,
//         pub h4: u64,
//         pub h5: u64,
//         pub h6: u64,
//         pub h7: u64,
//         pub h8: u64,
//         pub h9: u64,
//         pub h10: u64,
//         pub h11: u64,
//         pub h12: u64,
//         pub h13: u64,
//         pub h14: u64,
//         pub other: ItemInner,
//         pub other1: ItemInner,
//         pub other2: ItemInner,
//         pub othe3: ItemInner,
//         pub other5: ItemInner,
//     }
// }

// codable! {
//     type Lenser = ItemInnerLenser;
//     #[derive(Debug, Default)]
//     struct ItemInner {
//         pub h11: u64,
//         pub h12: u64,
//         pub h13: u64,
//         pub h14: u64,
//         pub h15: u64,
//         pub h: u64,
//         pub h2: u64,
//         pub h3: u64,
//         pub h4: u64,
//         pub h5: u64,
//     }
// }

// // impl Codable for Item {
// //     const SIZE: usize = <Option::<u32> as Codable>::SIZE + <u64 as Codable>::SIZE;
// //     fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Item>) {
// //         self.foo.encode(&mut bytes.index_to(0));
// //         self.bar.encode(&mut bytes.index_to(<Option::<u32> as Codable>::SIZE));
// //     }

// //     fn decode(bytes: &buf::bytes::Ref<'_, Item>) -> Self where Self: Sized {
// //         Self {
// //             foo: Codable::decode(&bytes.index_to(0)),
// //             bar: Codable::decode(&bytes.index_to(<Option::<u32> as Codable>::SIZE))
// //         }
// //     }

// //     type Lenser = ItemLenser;
// //     fn lenser_from_root(root: lenser::Root<Self>) -> Self::Lenser {
// //         // let hey = Cool::SIZE;
// //         // Cool::lenser().wow();
// //         ItemLenser(root)
// //     }
// // }

// // codable! {
// //     pub type Lenser = CoolLenser;

// //     #[derive(Clone, Debug)]
// //     pub struct Cool {
// //         pub cool: u32,
// //         pub idk: u64,
// //         pub wow: Option<u32>,
// //     }
// // }
