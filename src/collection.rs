use std::{fs::File, path::Path};
use fmmap::{MmapFileExt as _, MmapFileMut, MmapFileMutExt as _};
use crate::codable::{self, BufMut};

use super::{entry, Codable, Lens};

#[derive(Debug)]
pub enum GetError<DecodeError> {
    Fmmap(fmmap::error::Error),
    Decode(DecodeError)
}

#[derive(Debug)]
pub enum AddError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum NewError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum OpenError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

pub struct Value<Entry> {
    file: File,
    next_entry_id: entry::Id<Entry>,
    file_map: MmapFileMut,
    entry_size: usize,
    margin: u64,
    max_margin: u64,
}

impl<Entry: Codable> Value<Entry> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NewError> {
        type E = NewError;
        let file = File::create(&path).map_err(E::Io)?;
        file.set_len(8).map_err(E::Io)?;
        let mut file_map = fmmap::MmapFileMut::open(path).map_err(E::Fmmap)?;
        
        let next_entry_id = entry::Id::zero();
        file_map.write_u64(next_entry_id.as_u64(), 0).map_err(E::Fmmap)?;
        Ok(Self {
            margin: 0,
            max_margin: 1000,
            file,
            file_map,
            next_entry_id,
            entry_size: Entry::size(),
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, OpenError> {
        type E = OpenError;
        let file = File::open(&path).map_err(E::Io)?;
        let file_map = fmmap::MmapFileMut::open(path).map_err(E::Fmmap)?;
        let next_entry_id = entry::Id::from_u64(file_map.read_u64(0).map_err(E::Fmmap)?);
        Ok(Self {
            margin: 0,
            max_margin: 1000,
            file,
            file_map,
            next_entry_id,
            entry_size: Entry::size(),
        })
    }

    fn entry_lens_offset<T>(&self, id: entry::Id<Entry>, lens: Lens<Entry, T>) -> usize {
        8 + self.entry_size * id.as_usize() + lens.offset()
    }

    fn entry_offset(&self, id: entry::Id<Entry>) -> usize {
        8 + self.entry_size * id.as_usize()
    }

    pub fn buf_ref<T: Codable>(&self, id: entry::Id<Entry>, lens: Lens<Entry, T>) -> Result<codable::BufRef<'_, T>, fmmap::error::Error> {
        Ok(codable::BufRef::new(self.file_map.bytes(self.entry_lens_offset(id, lens), T::size())?))
    }

    pub fn buf_mut<T: Codable>(&mut self, id: entry::Id<Entry>, lens: Lens<Entry, T>) -> Result<codable::BufMut<'_, T>, fmmap::error::Error> {
        Ok(codable::BufMut::new(self.file_map.bytes_mut(self.entry_lens_offset(id, lens), T::size())?))
    }

    pub fn get<T: Codable>(&self, id: entry::Id<Entry>, lens: Lens<Entry, T>) -> Result<T, GetError<T::DecodeError>> {
        let bytes = self.file_map.bytes(self.entry_lens_offset(id, lens), T::size()).map_err(GetError::Fmmap)?;
        T::decode(bytes).map_err(GetError::Decode)
    }

    pub fn set<'a, T: Codable>(&'a mut self, id: entry::Id<Entry>, lens: Lens<Entry, T>, value: &impl codable::Write<T>) -> Result<BufMut<'a, T>, fmmap::error::Error> {
        let mut buf_mut = self.buf_mut(id, lens)?;
        buf_mut.set(value);
        Ok(buf_mut)
    }

    pub fn add(&mut self, entry: &impl codable::Write<Entry>) -> Result<entry::Id<Entry>, AddError> {
        type E = AddError;
        let id = self.next_entry_id;
        if self.margin == 0 {
            let new_size = self.entry_offset(id + self.max_margin + 2) as u64;
            self.file.set_len(new_size).map_err(E::Io)?;
            self.file_map.truncate(new_size).map_err(E::Fmmap)?;
            self.margin = self.max_margin + 1;
        }
        self.margin -= 1;
        self.set(id, Lens::TO_SELF, entry).map_err(E::Fmmap)?;
        self.next_entry_id = self.next_entry_id.succ();
        self.file_map.write_u64(self.next_entry_id.as_u64(), 0).map_err(E::Fmmap)?;
        Ok(id)
    }
}