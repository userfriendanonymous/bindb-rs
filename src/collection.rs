use std::{fs::File, path::Path};
use fmmap::{MmapFileExt as _, MmapFileMut, MmapFileMutExt as _};
use crate::codable::{self, Buf as _};
use super::{entry, Codable, Lens};

#[derive(Debug)]
pub enum GetError<T: Codable> {
    Fmmap(fmmap::error::Error),
    Decode(T::DecodeError)
}

#[derive(Debug)]
pub enum AddError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum RemoveLastError {
    Fmmap(fmmap::error::Error),
    Io(std::io::Error),
}

#[derive(Debug)]
pub enum SwapRemoveError {
    RemoveLastError(RemoveLastError),
    Fmmap(fmmap::error::Error)
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

    fn entry_lens_offset<T>(&self, lens: Lens<Entry, T>, id: entry::Id<Entry>) -> usize {
        self.entry_offset(id) + lens.offset()
    }

    fn entry_offset(&self, id: entry::Id<Entry>) -> usize {
        8 + self.entry_size * id.as_usize()
    }

    pub fn last_entry_id(&self) -> Option<entry::Id<Entry>> {
        match self.next_entry_id.as_u64() {
            0 => None,
            _ => Some(self.next_entry_id - 1)
        }
    }

    // region: Core functions.
    pub fn buf_ref<T: Codable>(&self, lens: Lens<Entry, T>, id: entry::Id<Entry>) -> Result<codable::BufRef<'_, T>, fmmap::error::Error> {
        Ok(codable::BufRef::new(self.file_map.bytes(self.entry_lens_offset(lens, id), T::size())?))
    }

    pub fn buf_mut<T: Codable>(&mut self, lens: Lens<Entry, T>, id: entry::Id<Entry>) -> Result<codable::BufMut<'_, T>, fmmap::error::Error> {
        Ok(codable::BufMut::new(self.file_map.bytes_mut(self.entry_lens_offset(lens, id), T::size())?))
    }

    // pub fn buf_mut_ptr<T: Codable>(&mut self, lens: Lens<Entry, T>, id: entry::Id<Entry>) -> Result<codable::BufMutPtr<T>, fmmap::error::Error> {
    //     Ok(codable::BufMutPtr::new(self.file_map.bytes_mut(self.entry_lens_offset(lens, id), T::size())? as *mut _))
    // }
    // endregion: Core functions.

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
        self.set(Lens::to_self(), id, entry).map_err(E::Fmmap)?;
        self.next_entry_id = self.next_entry_id.succ();
        self.file_map.write_u64(self.next_entry_id.as_u64(), 0).map_err(E::Fmmap)?;
        Ok(id)
    }

    pub fn remove_last(&mut self) -> Result<(), RemoveLastError> {
        type E = RemoveLastError;
        let id = self.next_entry_id.prev();
        if self.margin >= self.max_margin {
            let new_size = self.entry_offset(id) as u64;
            self.file.set_len(new_size).map_err(E::Io)?;
            self.file_map.truncate(new_size).map_err(E::Fmmap)?;
            self.margin = 0;
        }
        self.margin += 1;
        self.file_map.write_u64(self.next_entry_id.as_u64(), 0).map_err(E::Fmmap)?;
        Ok(())
    }

    // Convenience functions.
    pub fn get<T: Codable>(&self, lens: Lens<Entry, T>, id: entry::Id<Entry>) -> Result<T, GetError<T>> {
        self.buf_ref(lens, id).map_err(GetError::Fmmap)?.decode().map_err(GetError::Decode)
    }

    pub fn set<'a, T: Codable>(&'a mut self, lens: Lens<Entry, T>, id: entry::Id<Entry>, value: &impl codable::Write<T>) -> Result<(), fmmap::error::Error> {
        let mut buf_mut = self.buf_mut(lens, id)?;
        buf_mut.set(value);
        Ok(())
    }

    pub fn all_ids(&self) -> impl Iterator<Item = entry::Id<Entry>> {
        entry::id::Range(entry::Id::zero(), self.next_entry_id)
    }

    pub fn find<T: Codable>(
        &self,
        lens: Lens<Entry, T>,
        // ids: impl Iterator<Item = entry::Id<Entry>>,
        f: impl Fn(&T) -> bool,
    ) -> Result<Option<(entry::Id<Entry>, T)>, GetError<T>> {
        for id in self.all_ids() {
            let data = self.get(lens, id)?;
            if f(&data) {
                return Ok(Some((id, data)))
            }
        }
        Ok(None)
    }

    pub fn find_exact<'a, T: Codable>(
        &self,
        lens: Lens<Entry, T>,
        // ids: impl Iterator<Item = entry::Id<Entry>>,
        other: &'a impl codable::AsBuf<'a, T>,
    ) -> Result<Option<entry::Id<Entry>>, fmmap::error::Error> {
        let buf = other.as_buf();
        let buf = buf.to_ref();
        for id in self.all_ids() {
            let data = self.buf_ref(lens, id)?;
            if buf == data {
                return Ok(Some(id))
            }
        }
        Ok(None)
    }

    pub fn copy<T: Codable>(&mut self, lens: Lens<Entry, T>, src_id: entry::Id<Entry>, dst_id: entry::Id<Entry>) -> Result<(), fmmap::error::Error> {
        let mut dst = unsafe { self.buf_mut(lens, dst_id)?.detach() };
        let src = self.buf_ref(lens, src_id)?;
        dst.set(&src);
        Ok(())
    }

    pub fn swap<T: Codable>(&mut self, lens: Lens<Entry, T>, a_id: entry::Id<Entry>, b_id: entry::Id<Entry>) -> Result<(), fmmap::error::Error> {
        if a_id != b_id {
            let mut a = unsafe { self.buf_mut(lens, a_id)?.detach() };
            let mut b = self.buf_mut(lens, b_id)?;
            a.swap(&mut b);
        }
        Ok(())
    }

    /// Swaps given entry with the last entry and removes it.
    /// WARNING: This method changes ID of the last entry in this collection. 
    /// You probably should only use this if this collection is used as a stack/set and not as a map.
    pub fn swap_remove(&mut self, id: entry::Id<Entry>) -> Result<(), SwapRemoveError> {
        type E = SwapRemoveError;
        if let Some(last_entry_id) = self.last_entry_id() {
            self.swap(Lens::to_self(), id, last_entry_id).map_err(E::Fmmap)?;
        }
        self.remove_last().map_err(E::RemoveLastError)?;
        Ok(())
    }
}