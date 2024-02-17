#![feature(generic_const_exprs)]
pub use lens::{Value as Lens, producer as lenser};
pub use codable::Instance as Codable;
pub use collection::Value as Collection;
pub use buf::{Instance as Buf, AsInstance as AsBuf};
use macros::macro_with_crate_path;
pub use macros;

pub mod codable;
pub mod entry;
pub mod lens;
pub mod collection;
pub mod buf;
mod private;
mod utils;

fn benchmark<R>(msg: &'static str, f: impl FnOnce() -> R) -> R {
    let start = std::time::Instant::now();
    let r = f();
    let elapsed = start.elapsed().as_millis();
    println!("{}: {} ms.", msg, elapsed);
    r
}

#[test]
fn benchmarks_million() {
    let mut db = benchmark("Open", || Collection::<Item>::new("./local/test_db").unwrap());

    let total = 1_000_000;

    benchmark("insert", || {
        for idx in 0 .. total {
            db.add(&Item {
                foo: Some(idx as u32),
                bar: idx,
            }).unwrap();
        }
    });

    benchmark("read", || {
        for idx in 0 .. total {
            db.get(Lens::FULL, entry::Id::from(idx)).unwrap();
        }
    });

    benchmark("update", || {
        for idx in 0 .. total {
            db.set(Lens::FULL, entry::Id::from(idx), &Item {
                foo: Some(idx as u32),
                bar: idx,
            }).unwrap();
        }
    });
}


// #[test]
fn benchmarks_large() {
    // let mut file = File::create("./local/len_test").unwrap();
    // let len = 1321528398;
    // file.set_len(8 + Item::SIZE as u64 * len).unwrap();
    // let mut file_map = MmapFileMut::open("./local/len_test").unwrap();
    // file_map.write_u64(len, 0).unwrap();
    let mut db = benchmark("Open", || Collection::<Item>::open("./local/len_test").unwrap());

    let total = 1321528398;

    benchmark("Some ops", || {
        db.set(Lens::FULL, entry::Id::from_u64(0), &Item {
            bar: 1,
            foo: Some(1)
        }).unwrap();

        db.set(Lens::FULL, entry::Id::from_u64(total - 2), &Item {
            bar: 999,
            foo: Some(999)
        }).unwrap();

        db.copy(Lens::FULL, entry::Id::from_u64(total - 2), entry::Id::from_u64(1)).unwrap();

        let buf1 = db.buf_ref(Lens::FULL, entry::Id::from_u64(0)).unwrap();
        let buf2 = db.buf_ref(Lens::FULL, entry::Id::from_u64(total - 2)).unwrap();
        println!("#0: {:?}", buf1.decode());
        println!("#(total - 2): {:?}", buf2.decode());
        println!("#1: {:?}", db.get(Lens::FULL, entry::Id::from_u64(1)));
    });

    // db.add(&Item {
    //     foo: None,
    //     bar: 20,
    // }).unwrap();

    // benchmark("insert", || {
    //     for i in 0 .. total {
    //         let id = db.add(&Item {
    //             foo: Some(i as f32 + 0.53),
    //             bar: i
    //         }).unwrap();
    //     }
    // });
    // db.set(Id(5), Item::foo(), &None).unwrap();

    let items_count = 1_000_000;

    benchmark("set", || {
        for i in 0 .. items_count {
            db.set(Item::lenser().bar(), entry::Id::from_u64(i), &i).unwrap();
        }
    });

    benchmark("read", || {
        for i in 0 .. items_count {
            db.get(Lens::FULL, entry::Id::from(i)).unwrap();
        }
    });

    let item = benchmark("find", || {
        db.find(Item::lenser().bar(), |&x| x >= items_count).unwrap()
    });

    let item = benchmark("find_exact", || {
        db.find_exact(Item::lenser().bar(), &items_count).unwrap()
    });

    // let item = benchmark("find_full", || {
    //     db.find(Lens::FULL, |x| x.bar >= total - 10).unwrap()
    // });

    // benchmark("set", || {
    //     for i in 0 .. total {
    //         db.set(Lens::FULL, entry::Id::from(i), &Item {
    //             foo: None,
    //             bar: 13490345
    //         }).unwrap();
    //     }
    // });

}

struct ItemLenser(lenser::Root<Item>);

impl ItemLenser {
    fn spawn<T>(offset: usize) -> Lens<Item, T> {
        Item::lenser().0.spawn(offset)
    }

    fn foo(&self) -> Lens<Item, Option<u32>> {
        self.0.spawn(0)
    }

    fn bar(&self) -> Lens<Item, u64> {
        self.0.spawn(<Option<u32>>::SIZE)
    }
}

#[derive(Debug)]
struct Item {
    pub foo: Option<u32>,
    pub bar: u64,
}

impl Codable for Item {
    const SIZE: usize = <Option::<u32> as Codable>::SIZE + <u64 as Codable>::SIZE;
    fn encode(&self, bytes: &mut buf::bytes::Mut<'_, Item>) {
        self.foo.encode(&mut bytes.index_to(0));
        self.bar.encode(&mut bytes.index_to(<Option::<u32> as Codable>::SIZE));
    }

    fn decode(bytes: &buf::bytes::Ref<'_, Item>) -> Self where Self: Sized {
        Self {
            foo: Codable::decode(&bytes.index_to(0)),
            bar: Codable::decode(&bytes.index_to(<Option::<u32> as Codable>::SIZE))
        }
    }

    type Lenser = ItemLenser;
    fn lenser_from_root(root: lenser::Root<Self>) -> Self::Lenser {
        let hey = Cool::SIZE;
        Cool::lenser().wow();
        ItemLenser(root)
    }
}

macro_with_crate_path!{macros::derive_codable; codable}

codable! {
    pub type Lenser = CoolLenser;

    #[derive(Clone, Debug)]
    pub struct Cool {
        pub cool: u32,
        pub idk: u64,
        pub wow: Option<u32>,
    }
}
