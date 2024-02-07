use std::{fs::File, io::{Seek, Write}, marker::PhantomData, path::Path, sync::RwLock};

use fmmap::{MmapFileExt as _, MmapFileMut, MmapFileMutExt};
use memmap::Mmap;
pub use lens::Value as Lens;
pub use codable::Instance as Codable;
pub use collection::Value as Collection;

pub mod codable;
pub mod entry;
pub mod lens;
pub mod collection;
mod private;

fn benchmark<R>(msg: &'static str, f: impl FnOnce() -> R) -> R {
    let start = std::time::Instant::now();
    let r = f();
    let elapsed = start.elapsed().as_millis();
    println!("{}: {} ms.", msg, elapsed);
    r
}

fn main() {
    let mut db = benchmark("Open", || Collection::new("test_db").unwrap());

    let total = 1000000;

    // db.add(&Item {
    //     foo: None,
    //     bar: 20,
    // }).unwrap();

    benchmark("insert", || {
        for i in 0 .. total {
            let id = db.add(&Item {
                foo: Some(i as f32 + 0.53),
                bar: i
            }).unwrap();
        }
    });
    // db.set(Id(5), Item::foo(), &None).unwrap();

    benchmark("read", || {
        for i in 0 .. total {
            db.get(entry::Id::from(i), Lens::to_self()).unwrap();
        }
    });

    benchmark("set", || {
        for i in 0 .. total {
            db.set(entry::Id::from(i), Lens::to_self(), &Item {
                foo: None,
                bar: 13490345
            }).unwrap();
        }
    });

    let bar = db.get(entry::Id::from(0), Lens::to_self()).unwrap();
    println!("{bar:?}");
    
}

#[derive(Debug)]
struct Item {
    pub foo: Option<f32>,
    pub bar: u64,
}

impl Item {
    pub fn foo() -> Lens<Self, Option<f32>> {
        Lens::unsafe_new(0)
    }

    pub fn bar() -> Lens<Self, u64> {
        Lens::unsafe_new(<Option::<f32> as Codable>::size())
    }
}

impl Codable for Item {
    type DecodeError = String;

    fn size() -> usize {
        <Option::<f32> as Codable>::size() + <u64 as Codable>::size()
    }

    fn encode(&self, bytes: &mut [u8]) {
        let foo_size = <Option::<f32> as Codable>::size();
        self.foo.encode(&mut bytes[0 .. foo_size as usize]);
        self.bar.encode(&mut bytes[foo_size as usize ..]);
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
        let foo_size = <Option::<f32> as Codable>::size() as usize;
        Ok(Self {
            foo: Codable::decode(&bytes[0 .. foo_size]).map_err(|_| "foo".to_string())?,
            bar: Codable::decode(&bytes[foo_size ..]).map_err(|_| "bar".to_string())?
        })
    }
}






// pub trait Codable {
//     type DecodeError;

//     fn encode(&self, bytes: &mut [u8]);
//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized;

//     fn size() -> u64;
// }



// #[derive(Clone, Copy)]
// struct Lens<B, T> {
//     _marker: PhantomData<(B, T)>,
//     offset: u64,
// }

// impl<B> Lens<B, B> {
//     pub fn to_self() -> Self {
//         Self {
//             _marker: Default::default(),
//             offset: 0
//         }
//     }
// }

// impl<B, T> Lens<B, T> {
//     pub fn chain<OT>(self, other: Lens<T, OT>) -> Lens<B, OT> {
//         Lens {
//             _marker: Default::default(),
//             offset: self.offset + other.offset
//         }
//     }

//     pub fn new(offset: u64) -> Self {
//         Self {
//             _marker: Default::default(),
//             offset
//         }
//     }
// }


// #[derive(Debug)]
// pub enum GetError<DecodeError> {
//     Fmmap(fmmap::error::Error),
//     Decode(DecodeError)
// }

// struct Database<Entry> {
//     file: File,
//     next_entry_id: Id,
//     file_map: MmapFileMut,
//     entry_size: u64,
//     margin: u64,
//     max_margin: u64,
//     _marker: PhantomData<Entry>
// }

// impl<Entry: Codable> Database<Entry> {
//     pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, fmmap::error::Error> {
//         let mut file = File::create(&path).unwrap();
//         file.set_len(8).unwrap();
//         let mut file_map = fmmap::MmapFileMut::open(path)?;
        
//         let next_entry_id = Id::zero();
//         file_map.write_u64(next_entry_id.0, 0)?;
//         Ok(Self {
//             margin: 0,
//             max_margin: 1000,
//             file,
//             file_map,
//             next_entry_id,
//             entry_size: Entry::size(),
//             _marker: Default::default()
//         })
//     }

//     pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, fmmap::error::Error> {
//         let file = File::create(&path).unwrap();
//         let file_map = fmmap::MmapFileMut::open(path)?;
//         let next_entry_id = Id(file_map.read_u64(0)?);
//         Ok(Self {
//             margin: 0,
//             max_margin: 1000,
//             file,
//             file_map,
//             next_entry_id,
//             entry_size: Entry::size(),
//             _marker: Default::default()
//         })
//     }

//     fn entry_lens_offset<T>(&self, id: Id, lens: Lens<Entry, T>) -> u64 {
//         8 + self.entry_size * id.0 + lens.offset
//     }

//     fn entry_offset(&self, id: Id) -> u64 {
//         8 + self.entry_size * id.0
//     }

//     pub fn get<T: Codable>(&self, id: Id, lens: Lens<Entry, T>) -> Result<T, GetError<T::DecodeError>> {
//         let bytes = self.file_map.bytes(self.entry_lens_offset(id, lens) as usize, T::size() as usize).map_err(GetError::Fmmap)?;
//         T::decode(bytes).map_err(GetError::Decode)
//     }

//     pub fn set<T: Codable>(&mut self, id: Id, lens: Lens<Entry, T>, value: &T) -> Result<(), fmmap::error::Error> {
//         let bytes = self.file_map.bytes_mut(self.entry_lens_offset(id, lens) as usize, T::size() as usize)?;
//         value.encode(bytes);
//         Ok(())
//     }

//     pub fn add(&mut self, entry: &Entry) -> Result<Id, fmmap::error::Error> {
//         let id = self.next_entry_id;
//         if self.margin == 0 {
//             let new_size = self.entry_offset(Id(id.0 + self.max_margin + 2));
//             self.file.set_len(new_size).unwrap();
//             self.file_map.truncate(new_size)?;
//             self.margin = self.max_margin + 1;
//         }
//         self.margin -= 1;
//         self.set(id, Lens::to_self(), entry)?;
//         self.next_entry_id = self.next_entry_id.succ();
//         self.file_map.write_u64(self.next_entry_id.0, 0)?;
//         Ok(id)
//     }
// }

// #[derive(Clone, Copy, Debug)]
// struct Id(u64);

// impl Id {
//     pub fn zero() -> Self {
//         Self(0)
//     }

//     pub fn succ(&self) -> Self {
//         Self(self.0 + 1)
//     }

//     pub fn next(&mut self) -> Self {
//         let value = self.clone();
//         self.0 += 1;
//         value
//     }
// }

// impl Codable for u32 {
//     type DecodeError = ();

//     fn size() -> u64 {
//         4
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
//     }
// }

// impl Codable for u64 {
//     type DecodeError = ();

//     fn size() -> u64 {
//         8
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
//     }
// }

// impl Codable for i32 {
//     type DecodeError = ();

//     fn size() -> u64 {
//         4
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         bytes.copy_from_slice(&self.to_le_bytes())
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
//     }
// }

// impl Codable for i64 {
//     type DecodeError = ();

//     fn size() -> u64 {
//         8
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         Ok(i64::from_le_bytes(bytes.try_into().unwrap()))
//     }
// }

// impl Codable for f32 {
//     type DecodeError = ();

//     fn size() -> u64 {
//         4
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         Ok(f32::from_le_bytes(bytes.try_into().unwrap()))
//     }
// }

// impl Codable for f64 {
//     type DecodeError = ();

//     fn size() -> u64 {
//         8
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         bytes.copy_from_slice(&self.to_le_bytes());
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         Ok(f64::from_le_bytes(bytes.try_into().unwrap()))
//     }
// }

// #[derive(Clone, Debug)]
// pub enum OptionDecodeError<ChildDecodeError> {
//     InvalidOption,
//     Child(ChildDecodeError),
// }

// impl<T: Codable> Codable for Option<T> {
//     type DecodeError = OptionDecodeError<T::DecodeError>;

//     fn size() -> u64 {
//         T::size() + 1
//     }

//     fn encode(&self, bytes: &mut [u8]) {
//         match self {
//             Some(v) => {
//                 bytes[0] = 1;
//                 v.encode(&mut bytes[1..]);
//             }
//             None => bytes[0] = 0
//         };
//     }

//     fn decode(bytes: &[u8]) -> Result<Self, Self::DecodeError> where Self: Sized {
//         match bytes[0] {
//             0 => Ok(None),
//             1 => T::decode(&bytes[1..]).map_err(OptionDecodeError::Child).map(Some),
//             _ => Err(OptionDecodeError::InvalidOption)
//         }
//     }
// }